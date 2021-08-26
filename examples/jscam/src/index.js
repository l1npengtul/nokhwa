import init, { requestPermissions, queryConstraints, queryCameras } from 'nokhwa';

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

deviceButton.addEventListener("click", function (event) {
    deviceList.innerHTML = "";
    queryCameras().then(
        ok => {
            ok.forEach((element) => {
                var new_list_element = document.createElement("li");
                new_list_element.innerHTML = element.toString();
                deviceList.appendChild(new_list_element);
            })
        },
        err => {
            deviceLabel.innerHTML = "device list: error: " + err.toString();
         }
    )
});
