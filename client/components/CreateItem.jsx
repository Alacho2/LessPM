
// This component should create the VaultItem.
// It should contain the username
// It should contain the password
// It should not generate an ID because that's Mongo's job.
// It should OFFER to allow the user to write their own password
// Or generate a strong password of sorts.
// Finally, to store the password, it should prompt for the auth
// And then ship the info to the server on success
// Server should AGAIN validate the token that the user is authenticated.

// All the information is sent with the auth finish request.
// That should deal with everything that needs to be in terms of the
// server accepting info.
import { Base64 } from "js-base64";
import { useState } from "react";
import GeneratePassword from "./GeneratePassword.jsx";
import authenticate from "./Authenticate.jsx";
const BASE_URL = "https://localhost:3000/";
const START_PASSWORD_CREATION_URL = `${BASE_URL}fido/start_password_creation`;
const END_PASSWORD_CREATION_URL = `${BASE_URL}fido/end_password_creation`;
const AUTH_HEADER = 'authorization';

// PROPS:
// setSection = (SECTION) => void,
// sections: SECTIONS,

const CreateItem = (props) => {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [website, setWebsite] = useState("");
  const [checked, setChecked] = useState(true);

  // There is absolutely no guarantee that the server accepts what I am about
  // to do. But I am SURE as hell going to give it a try.
  const tryToCreateAnEntry = async (event) => {
    event.stopPropagation();
    event.preventDefault();

    const user_name = "bjoggii";

    const startPasswordCreation = await fetch(START_PASSWORD_CREATION_URL, {
      method: "POST",
      credentials: "include",
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({name: user_name})
    });

    if (startPasswordCreation.status !== 200) {
      console.log("We booped out");
      return; // exit early
    }

    const credentials = await startPasswordCreation.json();
    const authToken = startPasswordCreation.headers.get(AUTH_HEADER);

    const{ challenge, allowCredentials } = credentials.publicKey;

    credentials.publicKey.challenge = Base64.toUint8Array(challenge);
    credentials.publicKey.allowCredentials = allowCredentials.map(listItem => ({
      ...listItem,
      id: Base64.toUint8Array(listItem.id),
    }));

    const credentialsKeys
      = await navigator.credentials.get({publicKey: credentials.publicKey});

    if (!credentialsKeys) {
      return;
    }

    const {
      authenticatorData,
      clientDataJSON,
      signature,
    } = credentialsKeys.response;

    const uint8AuthData
      = Base64.fromUint8Array(new Uint8Array(authenticatorData));
    const uint8ClientDataJSON
      = Base64.fromUint8Array(new Uint8Array(clientDataJSON));
    const uint8Signature = Base64.fromUint8Array(new Uint8Array(signature));

    const credentialsToSend = {
      id: credentialsKeys.id,
      rawId: Base64.fromUint8Array(new Uint8Array(credentialsKeys.rawId), true),
      type: credentialsKeys.type,
      response: {
        authenticatorData: uint8AuthData,
        clientDataJSON: uint8ClientDataJSON,
        signature: uint8Signature,
      },
    };

    const userDataToSend = {
      website,
      password,
      username,
    };

    const body = {
      credentials: credentialsToSend,
      userData: userDataToSend,
      process: "creation",
    };

    const authorized = await fetch(END_PASSWORD_CREATION_URL, {
      method: "POST",
      headers: {
        'Authorization': authToken,
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify(body),
    });

    if (authorized.status !== 200) {
      console.log("That didn't work");
      return;
    }

    console.log("Check the server");


    // Now if this goes through, I want eventually CALL the setSections
    // props.setSection(props.sections.home)
  };

  // I want the input and password to either be a grid or a flexbox

  // Also, this should have a check about token which makes it just stop
  // rendering if there is no token. Fuckers. Or maybe everything is there
  // It just doesn't do anything.
  // Whichever.

  return (
    <div className="mt-4 bg-light border rounded create-item">
      <div className="mx-3 my-3">
        <div className="controller mb-3">
          <div className="havard-class">
            <div className="mb-3 input-group">
              <div className="input-group-prepend">
                <span className="input-group-text" id="basic-addon1">@</span>
              </div>
              <input
                type="text"
                className="form-control"
                placeholder="Username"
                aria-label="Username"
                onChange={({target}) => setUsername(target.value)}
                aria-describedby="basic-addon1" value={username} />
            </div>
            <div className="mb-3 input-group">
              <input
                type="text"
                className="form-control"
                placeholder="Website"
                aria-label="Website"
                onChange={({target}) => setWebsite(target.value)}
                aria-describedby="basic-addon1" value={website} />
            </div>
            <div className="mt-5"></div>
            <button
              onClick={tryToCreateAnEntry}
              type="button"
              className="create-button btn btn-primary">
              Excelsior
            </button>
          </div>

          <div className="havard-class password">
            <div className="input-group">

              <div className="input-group-prepend">
                <div className="input-group-text">
                  <input
                    className="checkbox-controlled-height"
                    type="checkbox"
                    defaultChecked={true}
                    onChange={() => setChecked(!checked)}
                    aria-label="Checkbox for following text input" />
                </div>
              </div>
              <input
                type="text" // This depends on the checkbox
                className="form-control"
                value={password}
                onChange={({target}) => setPassword(target.value) }
                placeholder="P4sSwØrd"
                disabled={checked}
                aria-label="Text input with checkbox" />
              <div
                onClick={() => navigator.clipboard.writeText(password)}
                className="input-group-append">
              </div>
            </div>
            {checked
              ? <GeneratePassword setPassword={setPassword}/>
              : <small>You realize that this is not as secure, right?</small>
            }
          </div>
        </div>

      </div>
    </div>
  )
};

export default CreateItem;
