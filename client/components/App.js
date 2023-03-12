// import { useState } from 'react';
import { Base64 } from 'js-base64';

export function App() {

  // We'll start by making a simple function to do authentication
  // Then a simple function to do login
  // We'll have to figure out how we are going to authenticate with what site
  // And how that should work. What info to send.
  // What info the backend expects.
  // Do we need a new key for everything?

  const authenticate = async () => {
    await fetch('https://localhost:3000/fido/start_registration', {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      },
      credentials: "include",
      method: "post",
      body: JSON.stringify({name: "bjoggii"})})
      .then(async resp => {
        const json = await resp.json();
        return {
          ...json, token: resp.headers.get('authorization')
        }
      }).then(async credentials => {
        credentials.publicKey.challenge = Base64.toUint8Array(credentials.publicKey.challenge);
        credentials.publicKey.user.id = Base64.toUint8Array(credentials.publicKey.user.id);

        await navigator.credentials.create({publicKey: credentials.publicKey});

        // const created = await navigator.credentials.create({publicKey: credentials.publicKey});
        // return {
        //   token: credentials.token,
        //   credentials: created,
        // };


      })/*.then(credentials_other => {
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
        })
      }); */
  };

  return (
    <>
      <p>Hello</p>
      <div onClick={authenticate}>Click me</div>
    </>
  );
  // return (<h1 onClick={() => setName("Eva")}>Hello {name}</h1>);
}
