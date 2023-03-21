import { Base64 } from "js-base64";
import { useState } from "react";
import getCredentialsBody from "./getCredentialsBody";
import performPostRequest from "./performPostRequest";

const BASE_URL = "https://localhost:3000/";
const START_AUTH_URL = `${BASE_URL}fido/start_authentication`;
const FINISH_AUTH_URL = `${BASE_URL}fido/finish_authentication`;
const AUTH_HEADER = 'authorization';

// PROPS:
// username: string
// setUsername: (username) => void,

const Login = (props) => {
  const [localUsername, setLocalUsername] = useState("");

  const login = async (username) => {
    if (!username.length) {
      return;
    }

    try {
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

      const body = await getCredentialsBody(credentials);

      if (!body) {
        return;
      }

      const {status} = await performPostRequest(FINISH_AUTH_URL, authToken, body);

      if (status !== 200) {
        return;
      }

      props.setUsername(username);
      props.setSection(props.sections.vault);
    } catch { /* Don't do anything */ }
  };

  // Parameters:
  // SyntheticKeyboardEvent<HTMLElement>
  const keyDownHandler = (event) => {
    if (event.key === "Enter") {
      login(localUsername)
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
                autoFocus
                onChange={({target}) => setLocalUsername(target.value)}
                className="form-control"
                aria-describedby="emailHelp"
                placeholder="Enter Username" />
            </div>
          </div>
          <div className="col-2">
            <button
              type="button"
              onClick={() => login(localUsername)}
              className="btn btn-primary">Login</button>
          </div>
        </div>
      </div>
    </div>
  )
};

export default Login;
