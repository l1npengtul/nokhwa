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
        scmSkip(deleteBuild: true, skipPattern: '.*\\[ci skip\\].*')
      }
    }

    stage('Cargo RustFMT') {
      agent {
        node {
          label 'ci_linux'
        }

      }
      steps {
        scmSkip(deleteBuild: true, skipPattern: '.*\\[ci skip\\].*')
        sh 'rustup update stable'
        sh 'cargo fmt --all -- --check'
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
            scmSkip(skipPattern: '.*\\[ci skip\\].*', deleteBuild: true)
            sh 'rustup update stable'
            sh 'cargo clippy --features "input-v4l, output-wgpu, test-fail-warning"'
          }
        }

        stage('Media Foundation') {
          agent {
            node {
              label 'ci_windows'
            }

          }
          steps {
            scmSkip(skipPattern: '.*\\[ci skip\\].*', deleteBuild: true)
            pwsh(script: 'rustup update stable', returnStatus: true)
            pwsh(script: 'cargo clippy --features "input-msmf, output-wgpu, test-fail-warning"', returnStatus: true)
          }
        }

        stage('AVFoundation') {
          steps {
            sh 'echo TODO'
          }
        }

        stage('libUVC') {
          agent {
            node {
              label 'ci_linux'
            }

          }
          steps {
            scmSkip(skipPattern: '.*\\[ci skip\\].*', deleteBuild: true)
            sh 'rustup update stable'
            sh 'cargo clippy --features "input-uvc, output-wgpu, test-fail-warning"'
          }
        }

        stage('OpenCV IPCamera') {
          agent {
            node {
              label 'ci_linux'
            }

          }
          steps {
            scmSkip(skipPattern: '.*\\[ci skip\\].*', deleteBuild: true)
            sh 'rustup update stable'
            sh 'cargo clippy --features "input-opencv, input-ipcam, output-wgpu, test-fail-warning"'
          }
        }

        stage('GStreamer') {
          agent {
            node {
              label 'ci_linux'
            }

          }
          steps {
            scmSkip(skipPattern: '.*\\[ci skip\\].*', deleteBuild: true)
            sh 'rustup update nightly'
            sh 'cargo clippy --features "input-gst, output-wgpu, test-fail-warning"'
          }
        }

        stage('JSCamera/WASM') {
          steps {
            scmSkip(skipPattern: '.*\\[ci skip\\].*', deleteBuild: true)
            sh 'rustup update stable'
            sh 'wasm-pack build --release -- --features "input-jscam, output-wasm,  test-fail-warning" --no-default-features'
            sh 'cargo clippy --features "input-jscam, output-wasm,  test-fail-warning" --no-default-features'
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
        scmSkip(skipPattern: '.*\\[ci skip\\].*', deleteBuild: true)
        sh 'rustup update nightly'
        sh 'cargo +nightly doc --features "docs-only, docs-nolink, docs-features, test-fail-warning" --no-deps --release'
      }
    }

  }
}