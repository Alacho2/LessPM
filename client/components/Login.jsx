import { Base64 } from "js-base64";
import { useState } from "react";
import getCredentialsBody from "./getCredentialsBody";

const BASE_URL = "https://localhost:3000/";
const START_AUTH_URL = `${BASE_URL}fido/start_authentication`;
const FINISH_AUTH_URL = `${BASE_URL}fido/finish_authentication`;
const AUTH_HEADER = 'authorization';

// PROPS:
// username: string

// The unique component needs a start end point and finish end point URL
// Should probably get the username from somewhere


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

    const body = await getCredentialsBody(credentials);

    if (!body) {
      return;
    }

    const authorized = await performPostRequest(FINISH_AUTH_URL, authToken, body);

    if (!authorized !== 200) {
      return;
    }

    console.log("Time for a dance");

  };

  return (
    <div onClick={() => login(props.username)}>Login</div>
  )
};

export default Login;
