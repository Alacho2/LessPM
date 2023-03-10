![u2f process](https://developers.yubico.com/U2F/Libraries/Using_a_library__1.png)

A description of the U2F library
https://developers.yubico.com/U2F/Libraries/Using_a_library.html

anyhow - if all else fails


https://github.com/grantila/u2f-api

UIDevice.current.identifierForVendor!.uuidString




fetch('http://localhost:8080/fido/start_registration', {    headers: {
'Accept': 'application/json',
'Content-Type': 'application/json'
},
method: "post", body: JSON.stringify({name: "bjoggi"})}).then(resp => resp.json()).then(
credentials =>  {
credentials.publicKey.challenge = new TextEncoder().encode(credentials.publicKey.challenge)
credentials.publicKey.user.id = new TextEncoder().encode(credentials.publicKey.user.id)
return navigator.credentials.create({publicKey: credentials.publicKey })
})

You can launch the browser with a uniquely generated ID that the server then 
checks
