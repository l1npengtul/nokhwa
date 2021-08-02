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

    stage('Clippy Build') {
      parallel {
        stage('V4L2') {
          agent {
            node {
              label 'ci_linux'
            }

          }
          steps {
            sh 'rustup update stable'
            sh 'cargo clippy --features "input-v4l"    -- -D warnings'
          }
        }

        stage('Media Foundation') {
          agent {
            node {
              label 'ci-agent-win10'
            }

          }
          steps {
            bat(script: 'rustup update stabke', encoding: 'UTF8')
            bat(script: 'cargo clippy --features "input-msmf" -- -D warnings', encoding: 'UTF8', returnStatus: true, returnStdout: true)
          }
        }

        stage('libUVC-Linux') {
          steps {
            sh '''rustup update stable
'''
            sh 'cargo clippy --features "input-uvc" -- -D warnings '
          }
        }

      }
    }

  }
}