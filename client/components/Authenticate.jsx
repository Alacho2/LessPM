import { Base64 } from "js-base64";

const BASE_URL = "https://localhost:3000/";
const START_REG_URL = `${BASE_URL}fido/start_registration`;
const FINISH_REG_URL = `${BASE_URL}fido/finish_registration`;
const AUTH_HEADER = 'authorization';

// PROPS:
// username: string
const Authenticate = (props) => {
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

  return (
    <div onClick={() => authenticate(props.username)}>Auth</div>
  )
};

export default Authenticate;
