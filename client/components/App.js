import {useState} from 'react';
import Authenticate from "./Authenticate.jsx";
import Login from "./Login.jsx";


const SECTIONS = Object.freeze({
  home: "HOME",
  other: "OTHER"
});

export function App() {
  const [section, setSection] = useState(SECTIONS.home);

  // We'll start by making a simple function to do authentication
  // Then a simple function to do login
  // We'll have to figure out how we are going to authenticate with what site
  // And how that should work. What info to send.
  // What info the backend expects.
  // Do we need a new key for everything?

  // I am not going to bother with a router. It is JUST as simple to render
  // Using sections
  const username = "bjoggii";

  const renderHome = () => {
    return (
      <div className="container">

      <p>Hello</p>
      <Authenticate username={username}/>
      <Login username={username}/>
      <p onClick={() => setSection(SECTIONS.other)}>Something</p>
    </div>
    )
  };

  const renderSomeOtherThing = () => {
    return (
      <p>Derp</p>
    )
  };


  // if (section === SECTIONS.home) {
  //   return renderHome();
  // }

  return (
    <>
      <div className="navbar navbar-light bg-light">
        <div className="container">
            <h1
              role="button"
              onClick={() => setSection(SECTIONS.home)}
              className="navbar-brand">
              LessPM
            </h1>
        </div>
      </div>
      {section === SECTIONS.home && renderHome()}
    </>
)
}
