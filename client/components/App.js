// import { useState } from 'react';
import { Base64 } from 'js-base64';
import Authenticate from "./Authenticate.jsx";
import Login from "./Login.jsx";

export function App() {

  // We'll start by making a simple function to do authentication
  // Then a simple function to do login
  // We'll have to figure out how we are going to authenticate with what site
  // And how that should work. What info to send.
  // What info the backend expects.
  // Do we need a new key for everything?

  const username = "bjoggii";

  return (
    <>
      <p>Hello</p>
      <Authenticate username={username} />
      <Login username={username} />
    </>
  );
  // return (<h1 onClick={() => setName("Eva")}>Hello {name}</h1>);
}
