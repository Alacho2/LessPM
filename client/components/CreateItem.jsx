import { useState } from "react";
import GeneratePassword from "./GeneratePassword.jsx";
import getCredentialsBody from "./getCredentialsBody";
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
    try {

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

      const credentialsToSend = await getCredentialsBody(credentials);

      if (!credentialsToSend) {
        return;
      }

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

      const authorized = await performPostRequest(END_PASSWORD_CREATION_URL, authToken, body);

      if (authorized !== 201) {
        return;
      }



      props.setSection(props.sections.vault)
    } catch { /* Don't do anything */ }
  };

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
