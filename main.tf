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
  name               = "knative-cluster"
  location           = "us-east1-b"
  initial_node_count = 1
  min_master_version = "latest"

  node_config {
    machine_type = "e2-small"
    spot         = true

    oauth_scopes = [
      "https://www.googleapis.com/auth/cloud-platform",
    ]
  }

  # Define autoscaling for the default node pool
  remove_default_node_pool = true

  network    = "default"
  subnetwork = "default"

  addons_config {
    http_load_balancing {
      disabled = false
    }
  }

  logging_service    = "none"
  monitoring_service = "none"

  depends_on = [
    google_project_service.gke,
    google_project_service.cloudapis,
    google_project_service.containerregistry,
  ]
}

# Define a separate node pool with autoscaling enabled
resource "google_container_node_pool" "primary_nodes" {
  name       = "primary-node-pool"
  location   = "us-east1-b"
  cluster    = google_container_cluster.knative_cluster.name
  node_count = 1

  node_config {
    machine_type = "e2-micro"
    spot         = true

    oauth_scopes = [
      "https://www.googleapis.com/auth/cloud-platform",
    ]
  }

  autoscaling {
    min_node_count = 1
    max_node_count = 6
  }
}