importScripts('https://www.gstatic.com/firebasejs/9.2.0/firebase-app-compat.js');
importScripts('https://www.gstatic.com/firebasejs/9.2.0/firebase-messaging-compat.js');

firebase.initializeApp({
    apiKey: "AIzaSyCwo0EWTJz_w-J1lUf9w9NcEBdLNmGUaIo",
    authDomain: "hot-or-not-feed-intelligence.firebaseapp.com",
    projectId: "hot-or-not-feed-intelligence",
    storageBucket: "hot-or-not-feed-intelligence.appspot.com",
    messagingSenderId: "82502260393",
    appId: "1:82502260393:web:390e9d4e588cba65237bb8"
});

const messaging = firebase.messaging();

export function get_token() {
    messaging.getToken({ vapidKey: 'BOmsEya6dANYUoElzlUWv3Jekmw08_nqDEUFu06aTak-HQGd-G_Lsk8y4Bs9B4kcEjBM8FXF0IQ_oOpJDmU3zMs' }).then((currentToken) => {
        if (currentToken) {
            // Send the token to your server and update the UI if necessary
            console.log('currentToken', currentToken);
            return currentToken;
        } else {
            // Show permission request UI
            console.log('No registration token available. Request permission to generate one.');
            // ...
        }
    }).catch((err) => {
        console.log('An error occurred while retrieving token. ', err);
        // ...
    });
}