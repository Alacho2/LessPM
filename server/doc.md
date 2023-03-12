![u2f process](https://developers.yubico.com/U2F/Libraries/Using_a_library__1.png)

A description of the U2F library
https://developers.yubico.com/U2F/Libraries/Using_a_library.html

anyhow - if all else fails


https://github.com/grantila/u2f-api

UIDevice.current.identifierForVendor!.uuidString




fetch('https://localhost:3000/fido/start_registration', {
headers: {
'Accept': 'application/json',
'Content-Type': 'application/json'
},
method: "post",
body: JSON.stringify({name: "bjoggi"})})
.then(async resp => {
const json = await resp.json();
return {
...json, token: resp.headers.get('authorization')
}
}).then(
async credentials =>  {
credentials.publicKey.challenge = new TextEncoder().encode(credentials.publicKey.challenge);
credentials.publicKey.user.id = new TextEncoder().encode(credentials.publicKey.user.id);

      const created = await navigator.credentials.create({publicKey: credentials.publicKey});
return {
token: credentials.token,
credentials: created,
};
}).then(credentials_other => {
const { credentials } = credentials_other;
console.log(credentials);

      const somebody = {
            id: credentials.id,
            rawId: Base64.fromUint8Array(new Uint8Array(credentials.id), true),
            type: credentials.type,
            response: {
                attestationObject: Base64.fromUint8Array(new Uint8Array(credentials.response.attestationObject), true),
                clientDataJSON: Base64.fromUint8Array(new Uint8Array(credentials.response.clientDataJSON), true),
            }
        };
      console.log(somebody);
    fetch('https://localhost:3000/fido/finish_registration', {
        headers: {
            'Authorization': credentials_other.token,
            'Accept': 'application/json',
            'Content-Type': 'application/json',
        },
        method: "post",
        body: JSON.stringify(somebody)
    })});

You can launch the browser with a uniquely generated ID that the server then 
checks

Privately signed certificate and private key for HTTPS
↠ openssl req -x509 -out certificate.pem -keyout privatekey.pem -newkey rsa:2048 -days 1024 -nodes -sha256 -subj "/C=IS/O=ReykjavikUniversity/CN=localhost" -extensions EXT -config domains.txt
Generating a 2048 bit RSA private key
............................................+++++
..............+++++
writing new private key to 'privatekey.pem'



async function login() {
await fetch('https://localhost:3000/fido/start_authentication', {
method: "POST",
headers: {
'Accept': 'application/json',
'Content-Type': 'application/json',
},
body: JSON.stringify({name: "bjoggii"})
}).then(async resp => {
const json = await resp.json();
console.log(resp.headers.get('authorization'));
return {
...json,
token: resp.headers.get('authorization')
}
}).then(async credentials => {
credentials.publicKey.challenge = Base64.toUint8Array(credentials.publicKey.challenge);
credentials.publicKey.allowCredentials.forEach(listItem => {
listItem.id = Base64.toUint8Array(listItem.id);            
});
const gotten = await navigator.credentials.get({ publicKey: credentials.publicKey });
return { token: credentials.token, credentials: gotten }
}).then(credentials_other => {
const { credentials } = credentials_other;

        console.log(credentials);

        const body = {
                id: credentials.id,
                rawId: Base64.fromUint8Array(new Uint8Array(credentials.rawId), true),
                type: credentials.type,
                response: {
                    authenticatorData: Base64.fromUint8Array(new Uint8Array(credentials.response.authenticatorData), true),
                    clientDataJSON: Base64.fromUint8Array(new Uint8Array(credentials.response.clientDataJSON), true),
                    signature: Base64.fromUint8Array(new Uint8Array(credentials.response.signature), true),
                    // userHandle: credentials.response.userHandle
                },
            }

        console.log(body);
        fetch("https://localhost:3000/fido/finish_authentication", {
            method: "POST",
            headers: {
              'Authorization': credentials_other.token,  
              'Accept': 'application/json',
              'Content-Type': 'application/json'
            },
            body: JSON.stringify(body)
        }).then(response => {
            if (response.ok) {
                console.log("Damdam");
            } else {
                console.log("Derp");
            }
        })
    })
}

async function authenticate_1() {
await fetch('https://localhost:3000/fido/start_registration', {
headers: {
'Accept': 'application/json',
'Content-Type': 'application/json'
},
method: "post",
body: JSON.stringify({name: "bjoggii"})})
.then(async resp => {
const json = await resp.json();
return {
...json, token: resp.headers.get('authorization')
}
}).then(
async credentials => {
credentials.publicKey.challenge = Base64.toUint8Array(credentials.publicKey.challenge);
credentials.publicKey.user.id = Base64.toUint8Array(credentials.publicKey.user.id);

      const created = await navigator.credentials.create({publicKey: credentials.publicKey});
return {
token: credentials.token,
credentials: created,
};
}).then(credentials_other => {
const { credentials } = credentials_other;
fetch('https://localhost:3000/fido/finish_registration', {
headers: {
'Authorization': credentials_other.token,
'Accept': 'application/json',
'Content-Type': 'application/json',
},
method: "post",
body: JSON.stringify({
id: credentials.id,
rawId: Base64.fromUint8Array(new Uint8Array(credentials.id), true),
type: credentials.type,
response: {
attestationObject: Base64.fromUint8Array(new Uint8Array(credentials.response.attestationObject), true),
clientDataJSON: Base64.fromUint8Array(new Uint8Array(credentials.response.clientDataJSON), true),
}
})
})});
}

Exposing the Auth token

List of Stuff To Do:
[]: Encryption on the passwords
[]: Stop storing the keys in memory
[]: Password Creator
[]: UI

