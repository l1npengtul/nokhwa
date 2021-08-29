import init, { requestPermissions, queryConstraints, queryCameras, NokhwaCamera, CameraConstraints, CameraConstraintsBuilder, CameraFacingMode, CameraResizeMode } from 'nokhwa';

async function start() {
    await init();
}
start();

const requestStatus = document.getElementById("requestStatus");
const requestButton = document.getElementById("requestButton");

requestButton.addEventListener("click", function (event) {
    requestPermissions().then(
        ok => {
            requestStatus.innerHTML = "Granted :D";
        },
        err => {
            requestStatus.innerHTML = "Denied :( due to " + err.toString();
        }
    )
});

const constraintList = document.getElementById("constraintList");
const constraintButton = document.getElementById("constraintButton");

constraintButton.addEventListener("click", function (event) {
    constraintList.innerHTML = "";
    queryConstraints().forEach((element) => {
        var new_list_element = document.createElement("li");
        new_list_element.innerHTML = element.toString();
        constraintList.appendChild(new_list_element);
    })
});

const deviceLabel = document.getElementById("deviceLabel");
const deviceList = document.getElementById("deviceList");
const deviceButton = document.getElementById("deviceButton");
const deviceDropdown = document.getElementById("deviceDropdown");

deviceButton.addEventListener("click", function (event) {
    deviceList.innerHTML = "";
    deviceDropdown.innerHTML = "";
    queryCameras().then(
        ok => {
            ok.forEach((element) => {
                var new_list_element = document.createElement("li");
                new_list_element.innerHTML = "Name: " + element.HumanReadableName;
                deviceList.appendChild(new_list_element);

                var new_option = document.createElement("option");
                new_option.value = element.MiscString;
                new_option.innerHTML = element.HumanReadableName;
                deviceDropdown.appendChild(new_option);
            })
        },
        err => {
            deviceLabel.innerHTML = "device list: error: " + err.toString();
         }
    )
});

const deviceOpenButton = document.getElementById("deviceOpenButton");
const streamPlayLabel = document.getElementById("streamPlayLabel");
const streamPlayArea = document.getElementById("streamPlayArea");
var nokhwaCamera = undefined;

deviceOpenButton.addEventListener("click", function(event) {
    streamPlayArea.innerHTML = "";
    let constraints = (new CameraConstraintsBuilder()).buildCameraConstraints();
    nokhwaCamera = new NokhwaCamera(constraints).catch((err) => {console.error(err); return});
    nokhwaCamera = nokhwaCamera.then(
        (ok) => {
            nokhwaCamera = ok;
            nokhwaCamera.attachToElement("streamPlayArea", true);
            streamPlayLabel.innerHTML = "Stream Get!";
        },
        (err) => {
            console.error(err); 
            streamPlayLabel.innerHTML = "Error: " + err.toString();
            return;
        }
    );
});

const streamStopButton = document.getElementById("streamStopButton");

streamStopButton.addEventListener("click", function(event) {
    if (nokhwaCamera !== undefined) {
        nokhwaCamera.detachCamera().then(
            (ok) => {
                nokhwaCamera = undefined;
                streamPlayArea.innerHTML = "";
                streamPlayLabel.innerHTML = "Stream Detached...";
            },
            (err) => {
                console.error(err); 
                streamPlayLabel.innerHTML = "Error: " + err.toString();
                return;
            }
        )
    }
});

const streamCaptureImageButton = document.getElementById("streamCaptureImageButton");

function downloadURI(uri, name) {
    var link = document.createElement("a");
    link.download = name;
    link.href = uri;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
}
  

streamCaptureImageButton.addEventListener("click", function(event) {
    if (nokhwaCamera !== undefined) {
        let uri = nokhwaCamera.captureImageURI("image/jpeg", 0.75);
        downloadURI(uri, "capture.jpg");
    }
});
