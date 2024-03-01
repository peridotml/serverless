terraform {
  backend "gcs" {
    bucket = "atlantis-demo"
    prefix = "terraform"
  }

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "4.51.0"
    }
  }
}

provider "google" {
  project = "serverless-415915"
  region  = "us-east1"
  zone    = "us-east1-b"
}

resource "google_project_service" "gke" {
  service            = "container.googleapis.com"
  disable_on_destroy = false
}

resource "google_project_service" "cloudapis" {
  service            = "cloudapis.googleapis.com"
  disable_on_destroy = false
}

resource "google_project_service" "containerregistry" {
  service            = "containerregistry.googleapis.com"
  disable_on_destroy = false
}

resource "google_container_cluster" "knative_cluster" {
  name     = "knative-cluster"
  location = "us-east1-b"

  initial_node_count = 1
  min_master_version = "latest"

  node_config {
    machine_type = "e2-small" # Adjust based on your app's needs
    spot         = true       # Use Spot VMs for cost efficiency

    oauth_scopes = [
      "https://www.googleapis.com/auth/cloud-platform",
    ]
  }

  autoscaling {
    min_node_count = 1
    max_node_count = 6 # Adjust based on expected load
  }

  network    = "default"
  subnetwork = "default"

  addons_config {
    http_load_balancing {
      disabled = false
    }
  }

  # Assuming you're managing logging and monitoring elsewhere to control costs
  logging_service    = "none"
  monitoring_service = "none"

  depends_on = [google_project_service.gke, google_project_service.cloudapis, google_project_service.containerregistry]
}
