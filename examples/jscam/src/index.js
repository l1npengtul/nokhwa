import init, {requestPermissions} from 'nokhwa';

async function start() {
    await init();
}
start();

document.getElementById('requestButton').addEventListener('click', function(e) {
    requestPermissions()
} );
