// import { useState } from 'react';
import { Base64 } from 'js-base64';

const BASE_URL = "https://localhost:3000/";
const START_REG_URL = `${BASE_URL}fido/start_registration`;
const FINISH_REG_URL = `${BASE_URL}fido/finish_registration`;
const START_AUTH_URL = `${BASE_URL}fido/start_authentication`;
const FINISH_AUTH_URL = `${BASE_URL}fido/finish_authentication`;
const AUTH_HEADER = 'authorization';

export function App() {

  // We'll start by making a simple function to do authentication
  // Then a simple function to do login
  // We'll have to figure out how we are going to authenticate with what site
  // And how that should work. What info to send.
  // What info the backend expects.
  // Do we need a new key for everything?


  // Let's refactor this so it just feels a tonne better
  const authenticate = async (username) => {

    const registrationStart = await fetch(START_REG_URL, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      },
      credentials: "include",
      method: "post",
      body: JSON.stringify({name: username})
    });

    // Early return if the server reported a bad error
    if (registrationStart.status !== 200) {
      return;
    }

    const credentials = await registrationStart.json();
    const authToken = registrationStart.headers.get(AUTH_HEADER);

    // Server expects these to be Base64.
    credentials.publicKey.challenge
      = Base64.toUint8Array(credentials.publicKey.challenge);
    credentials.publicKey.user.id
      = Base64.toUint8Array(credentials.publicKey.user.id);

    const created = await navigator.credentials.create({
      publicKey: credentials.publicKey,
    });

    // something bad happened and we should NOT proceed.
    if (!created) {
      return;
    }

    console.log(created);

    // If everything checks out at this point, we can sniffing around at
    // attestation and finishing the reg process
    const { clientDataJSON, attestationObject } = created.response;

    const body = {
      id: created.id,
      rawId: Base64.fromUint8Array(new Uint8Array(created.id), true),
      type: created.type,
      response: {
        attestationObject: Base64.fromUint8Array(new Uint8Array(attestationObject), true),
        clientDataJSON: Base64.fromUint8Array(new Uint8Array(clientDataJSON), true),
      }
    };

    const finish = await fetch(FINISH_REG_URL, {
      headers: {
        'Authorization': authToken,
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
      method: 'post',
      body: JSON.stringify(body),
    });


    // We goofed up again and should stop the process.
    // Check the server.
    if (finish.status !== 200) {
      return;
    }

    console.log("The user got registered. Move on with your life");
  };

  const login = async (username) => {
      const startAuth = await fetch(START_AUTH_URL, {
        method: "POST",
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({name: username})
      });

    // We went and goofed up, check the server.
    if (startAuth.status !== 200) {
      return;
    }

    const credentials = await startAuth.json();
    const authToken = startAuth.headers.get(AUTH_HEADER);

    const { challenge, allowCredentials } = credentials.publicKey;

    credentials.publicKey.challenge = Base64.toUint8Array(challenge);
    credentials.publicKey.allowCredentials = allowCredentials.map(listItem => ({
      ...listItem,
      id: Base64.toUint8Array(listItem.id),
    }));

    const keys = await navigator.credentials.get({publicKey: credentials.publicKey});

    // Hold your horses, can you don't have the keys you need.
    if (!keys) {
      return;
    }

    const {
      authenticatorData,
      clientDataJSON,
      signature
    } = keys.response;

    const uint8AuthData
      = Base64.fromUint8Array(new Uint8Array(authenticatorData));
    const uint8ClientDataJSON =
      Base64.fromUint8Array(new Uint8Array(clientDataJSON));
    const uint8Signature = Base64.fromUint8Array(new Uint8Array(signature));

    const body = {
      id: keys.id,
      rawId: Base64.fromUint8Array(new Uint8Array(keys.rawId), true),
      type: keys.type,
      response: {
        authenticatorData: uint8AuthData,
        clientDataJSON: uint8ClientDataJSON,
        signature: uint8Signature
      }
    };

    const authenticated = await fetch(FINISH_AUTH_URL, {
      method: "POST",
      headers: {
        'Authorization': authToken,
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
    });

    if (authenticated.status !== 200) {
      console.log("Not authenticated");
      return;
    }

    console.log("Time for a dance");

  };

  return (
    <>
      <p>Hello</p>
      {}
      <div onClick={() => authenticate("bjoggii")}>Auth</div>
      <div onClick={() => login("bjoggii")}>Login</div>
      {/*<div onClick={() => window.open("https://google.com")}>Click me</div>*/}
    </>
  );
  // return (<h1 onClick={() => setName("Eva")}>Hello {name}</h1>);
}
