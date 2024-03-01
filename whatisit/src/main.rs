use axum::{extract::Multipart, response::{Html, IntoResponse}, routing::{get, post}, Json, Router};

mod model;
mod coco_classes;
use candle_core::{DType, Device, Tensor, IndexOp};
use candle_nn::{Module, VarBuilder};
use model::{Multiples, YoloV8};
use std::{collections::HashSet, io::Cursor};
use image::{io::Reader as ImageReader, DynamicImage};
use candle_transformers::object_detection::{non_maximum_suppression, Bbox, KeyPoint};
use serde::Serialize;


#[derive(Serialize)]
struct PredictResponse {
    objects: Vec<String>,
}

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/", get(handler))
                                   .route("/predict", post(predict));

    // run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
        <html>
        <head>
            <title>What is it?</title>
        </head>
        <body>
        
        <h2>Upload Image to Predict</h2>
        
        <!-- Simple HTML form -->
        <form id="uploadForm">
            <input type="file" id="imageInput" name="image" required>
            <button type="submit">Predict</button>
        </form>
        
        <!-- Placeholder for the prediction response -->
        <div id="predictionResult"></div>
        
        <script>
        // JavaScript to handle the form submission
        document.getElementById('uploadForm').addEventListener('submit', function(event) {
            event.preventDefault(); // Prevent the default form submission
        
            const formData = new FormData();
            const imageInput = document.getElementById('imageInput');
            if (imageInput.files.length > 0) {
                formData.append('image', imageInput.files[0]);
        
                // Perform the POST request using the Fetch API
                fetch('http://whatisit.default.ramblings-app.com/predict', {
                    method: 'POST',
                    body: formData,
                })
                .then(response => response.json()) // Assuming the response is JSON
                .then(data => {
                    // Display the response
                    document.getElementById('predictionResult').textContent = JSON.stringify(data);
                })
                .catch(error => console.error('Error:', error));
            }
        });
        </script>
        
        </body>
        </html>
    "#)
}

async fn predict(mut multipart: Multipart)  -> Result<Json<PredictResponse>, impl IntoResponse> {
    let mut original_image: Option<DynamicImage> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name().unwrap() == "image" {
            // Assuming the field containing the image is named "image"
            let data = field.bytes().await.unwrap();
            let img = ImageReader::new(Cursor::new(data)).with_guessed_format().unwrap().decode().unwrap();
            original_image= Some(img);
            break; // Break if you're only expecting one image
        }
    }

    if let Some(original_image) = original_image {

        // Load your model here and apply it to `data`
        let path = "models/yolov8n.safetensors";
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[path], DType::F32, &Device::Cpu).unwrap() };
        let multiples = Multiples::n();
        let model = YoloV8::load(vb, multiples, 80).unwrap();

        let (width, height) = {
            let w = original_image.width() as usize;
            let h = original_image.height() as usize;
            if w < h {
                let w = w * 640 / h;
                // Sizes have to be divisible by 32.
                (w / 32 * 32, 640)
            } else {
                let h = h * 640 / w;
                (640, h / 32 * 32)
            }
        };
        let image_t = {
            let img = original_image.resize_exact(
                width as u32,
                height as u32,
                image::imageops::FilterType::CatmullRom,
            );
            let data = img.to_rgb8().into_raw();
            Tensor::from_vec(
                data,
                (img.height() as usize, img.width() as usize, 3),
                &Device::Cpu,
            ).unwrap()
            .permute((2, 0, 1)).unwrap()
        };
        let image_t = (image_t.unsqueeze(0).unwrap().to_dtype(DType::F32).unwrap() * (1. / 255.)).unwrap();
        let predictions = model.forward(&image_t).unwrap().squeeze(0).unwrap();
        let objects = detect(&predictions, 0.25, 0.45);
        
        Ok(Json(PredictResponse { objects }))
    } else {
        Err((axum::http::StatusCode::BAD_REQUEST, "Oopsie doopsie! No image data was found"))
    }
}


pub fn detect(
    pred: &Tensor,
    confidence_threshold: f32,
    nms_threshold: f32,
) -> Vec<String>{
    let pred = pred.to_device(&Device::Cpu).unwrap();
    let (pred_size, npreds) = pred.dims2().unwrap();
    let nclasses = pred_size - 4;
    // The bounding boxes grouped by (maximum) class index.
    let mut bboxes: Vec<Vec<Bbox<Vec<KeyPoint>>>> = (0..nclasses).map(|_| vec![]).collect();
    // Extract the bounding boxes for which confidence is above the threshold.
    for index in 0..npreds {
        let pred = Vec::<f32>::try_from(pred.i((.., index)).unwrap()).unwrap();
        let confidence = *pred[4..].iter().max_by(|x, y| x.total_cmp(y)).unwrap();
        if confidence > confidence_threshold {
            let mut class_index = 0;
            for i in 0..nclasses {
                if pred[4 + i] > pred[4 + class_index] {
                    class_index = i
                }
            }
            if pred[class_index + 4] > 0. {
                let bbox = Bbox {
                    xmin: pred[0] - pred[2] / 2.,
                    ymin: pred[1] - pred[3] / 2.,
                    xmax: pred[0] + pred[2] / 2.,
                    ymax: pred[1] + pred[3] / 2.,
                    confidence,
                    data: vec![],
                };
                bboxes[class_index].push(bbox)
            }
        }
    }

    non_maximum_suppression(&mut bboxes, nms_threshold);

    let mut objects: HashSet<String> = HashSet::new();
    for (class_index, bboxes_for_class) in bboxes.iter().enumerate() {
        for _ in bboxes_for_class.iter() {
            objects.insert(coco_classes::NAMES[class_index].to_string());
        }
    }
    objects.into_iter().collect()

}