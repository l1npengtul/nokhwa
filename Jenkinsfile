pipeline {
  agent {
    node {
      label 'ci-agent-win10'
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
        stage('Linux') {
          agent {
            node {
              label 'ci-agent-linux-fedora'
            }

          }
          steps {
            sh 'rustup update stable'
            sh 'cargo clippy --all-features    -- -D warnings'
            catchError(catchInterruptions: true, stageResult: 'failure')
          }
        }

        stage('Windows') {
          agent {
            node {
              label 'ci-agent-win10'
            }

          }
          steps {
            bat(script: 'rustup update stabke', encoding: 'UTF8')
            bat(script: 'cargo clippy --all-features -- -D warnings', encoding: 'UTF8', returnStatus: true, returnStdout: true)
          }
        }

      }
    }

    stage('Rustfmt') {
      parallel {
        stage('Linux') {
          agent {
            node {
              label 'ci-agent-linux-fedora'
            }

          }
          steps {
            sh 'cargo +nightly fmt --all -- --write-mode diff'
          }
        }

        stage('Windows') {
          steps {
            bat(script: 'cargo +nightly fmt --all -- --write-mode diff', encoding: 'UTF8')
          }
        }

      }
    }

  }
}