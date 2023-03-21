import { Base64 } from "js-base64";
import {useState} from "react";

const BASE_URL = "https://localhost:3000/";
const START_REG_URL = `${BASE_URL}fido/start_registration`;
const FINISH_REG_URL = `${BASE_URL}fido/finish_registration`;
const AUTH_HEADER = 'authorization';

// PROPS:
// username: string
// setUsername: (string) => void
// setSection: (SECTION) => void,
// sections: SECTIONS
const Authenticate = (props) => {
  const [localUsername, setLocalUsername] = useState("");
  const authenticate = async (username) => {
    try {
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

    // If everything checks out at this point, we can sniff around at
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

    props.setUsername(username);
    props.setSection(props.sections.login);
    } catch { /* Don't do anything */ }
  };

  const keyDownHandler = (event) => {
    if (event.key === "Enter") {
      authenticate(localUsername)
    }
  };

  return (
    <div className="mt-4 bg-light position-relative border rounded">
      <div className="mx-5 my-5">
        <div className="row">
          <div className="col-10">
            <div className="input-group">
              <div className="input-group-prepend">
                <span className="input-group-text" id="basic-addon1">@</span>
              </div>
              <input
                onKeyDown={keyDownHandler}
                type="text"
                onChange={({target}) => setLocalUsername(target.value)}
                className="form-control"
                aria-describedby="emailHelp"
                placeholder="Enter Username" />
            </div>
          </div>
          <div className="col-2">
            <button
              type="button"
              onClick={() => authenticate(localUsername)}
              className="btn btn-primary">Register</button>
          </div>
        </div>
      </div>
    </div>
  )
};

export default Authenticate;
