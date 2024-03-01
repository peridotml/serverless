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

resource "null_resource" "example" {}
