terraform {
  backend "gcs" {
    bucket      = "atlantis-demo"
    prefix      = "terraform"
  }

  required_providers {
    google = {
      source = "hashicorp/google"
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