import { Base64 } from "js-base64";
import { useState } from "react";

const BASE_URL = "https://localhost:3000/";
const START_AUTH_URL = `${BASE_URL}fido/start_authentication`;
const FINISH_AUTH_URL = `${BASE_URL}fido/finish_authentication`;
const AUTH_HEADER = 'authorization';

// PROPS:
// username: string

const Login = (props) => {

  const [keys, setKeys] = useState([]);

  const login = async (username) => {
    const startAuth = await fetch(START_AUTH_URL, {
      method: "POST",
      credentials: 'same-origin',
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

    const credentialKeys = await navigator.credentials.get({publicKey: credentials.publicKey});

    console.log(credentialKeys);

    // Hold your horses, can you don't have the keys you need.
    if (!credentialKeys) {
      return;
    }

    const {
      authenticatorData,
      clientDataJSON,
      signature
    } = credentialKeys.response;

    const uint8AuthData
      = Base64.fromUint8Array(new Uint8Array(authenticatorData));
    const uint8ClientDataJSON =
      Base64.fromUint8Array(new Uint8Array(clientDataJSON));
    const uint8Signature = Base64.fromUint8Array(new Uint8Array(signature));

    const body = {
      id: credentialKeys.id,
      rawId: Base64.fromUint8Array(new Uint8Array(credentialKeys.rawId), true),
      type: credentialKeys.type,
      response: {
        authenticatorData: uint8AuthData,
        clientDataJSON: uint8ClientDataJSON,
        signature: uint8Signature
      }
    };

    // const httpReq = new XMLHttpRequest();
    // httpReq.open("POST", FINISH_AUTH_URL, true);
    // httpReq.setRequestHeader("Authorization", authToken);
    // httpReq.setRequestHeader("Accept", "application/json");
    // httpReq.setRequestHeader("Content-Type", "application/json");
    // httpReq.withCredentials = true;
    //
    // httpReq.onreadystatechange = () => {
    //   const headers = httpReq.getAllResponseHeaders();
    //   const arr = headers.trim().split(/[\r\n]+/);
    //   arr.forEach((value) => {
    //     console.log(value);
    //   });
    // };
    //
    // httpReq.onload = () => {
    //   console.log(httpReq.responseText);
    // };
    //
    // httpReq.send(JSON.stringify(body));

    const authenticated = await fetch(FINISH_AUTH_URL, {
      method: "POST",
      headers: {
        'Authorization': authToken,
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify(body),
    });

    if (authenticated.status !== 200) {
      console.log("Not authenticated");
      return;
    }

    const incomingCookie = authenticated.headers.get('cookie');

    const cookies = incomingCookie.split(";");
    console.log(cookies);
    const token = cookies[0];
    console.log(token);
    const [name, value] = token.split("=");

    document.cookie = name.trim()+"="+value.trim();

    // for(const i in cookies){
    //   const vals = cookies[i].split('=');
    //   console.log(vals[0], vals[1]);
    //   const name = vals.shift(0, 1).trim();
    //   console.log(name, vals.join());
    //   document.cookie = name+'='+vals.join('=');
    // }

    console.log("Time for a dance");


    await fetch(`${BASE_URL}fido/test`, {
      method: "GET",
      credentials: "include",
    })
  };

  return (
    <div onClick={() => login(props.username)}>Login</div>
  )
};

export default Login;
