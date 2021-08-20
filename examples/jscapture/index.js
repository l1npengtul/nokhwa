const requestButton = document.getElementById("request");
const permissionLabel = document.getElementById("permissionlabel");

requestButton.addEventListener("click", onRequestPermission);

function onRequestPermission(event) {
    requestPermissions().then(
        sucessful => {
            permissionLabel.innerHTML = "Permission granted :D"
        },
        rejection => {
            permissionLabel.innerHTML = "Permission denied :'("
        }
    )
}


