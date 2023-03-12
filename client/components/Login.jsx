import { Base64 } from "js-base64";

const BASE_URL = "https://localhost:3000/";
const START_AUTH_URL = `${BASE_URL}fido/start_authentication`;
const FINISH_AUTH_URL = `${BASE_URL}fido/finish_authentication`;
const AUTH_HEADER = 'authorization';

// PROPS:
// username: string

const Login = (props) => {

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
    <div onClick={() => login(props.username)}>Login</div>
  )
};

export default Login;
