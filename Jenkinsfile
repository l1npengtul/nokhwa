pipeline {
  agent {
    node {
      label 'ci_linux'
    }

  }
  stages {
    stage('Sanity Check') {
      steps {
        echo '$BUILD_TAG'
      }
    }

    stage('Cargo RustFMT') {
     agent {
       node {
         label 'ci_linux'
       }
     }

     steps {
      sh 'rustup update stable'
      sh "cargo fmt --all -- --check"
     }
    }

    stage('Build, Clippy') {
      parallel {
        stage('V4L2') {
          agent {
            node {
              label 'ci_linux'
            }
          }

          steps {
            sh 'rustup update stable'
            sh 'cargo build --features "input-v4l, output-wgpu" -- -D warnings'
            sh 'cargo clippy --features "input-v4l, output-wgpu" -- -D warnings'
          }
        }

        stage('Media Foundation') {
          agent {
            node {
              label 'ci-agent-win10'
            }
          }
          steps {
            bat(script: 'rustup update stable', encoding: 'UTF8')
            bat(script: 'cargo build --features "input-msmf, output-wgpu" -- -D warnings', encoding: 'UTF8', returnStatus: true, returnStdout: true)
            bat(script: 'cargo clippy --features "input-msmf, output-wgpu" -- -D warnings', encoding: 'UTF8', returnStatus: true, returnStdout: true)
          }
        }

        stage('AVFoundation') {
         steps {
          sh 'echo TODO'
         }
        }

        stage('libUVC Linux') {
          agent {
            node {
              label 'ci_linux'
            }
          }

          steps {
            sh 'rustup update stable'
            sh 'cargo build --features "input-uvc, output-wgpu" -- -D warnings'
            sh 'cargo clippy --features "input-uvc, output-wgpu" -- -D warnings '
          }
        }

        stage('OpenCV IPCamera Linux') {
          agent {
            node {
              label 'ci_linux'
            }
          }
          steps {
            sh 'rustup update stable'
            sh 'cargo build --features "input-opencv, input-ipcam, output-wgpu" -- -D warnings'
            sh 'cargo clippy --features "input-opencv, input-ipcam, output-wgpu" -- -D warnings '
          }
        }

        stage('GStreamer Linux') {
         agent {
           node {
             label 'ci_linux'
           }
         }
         steps {
           sh 'rustup update stable'
           sh 'cargo build --features "input-gst, output-wgpu" -- -D warnings'
           sh 'cargo clippy --features "input-gst, output-wgpu" -- -D warnings '
         }
        }
      }
    }
    stage('RustDOC') {
      agent {
        node {
          label 'ci_linux'
        }
      }

      steps {
        sh 'cargo doc --features "docs-only, docs-nolink" -- -D warnings'
      }
    }
  }
}