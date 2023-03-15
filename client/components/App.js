import {useState} from 'react';
import Authenticate from "./Authenticate.jsx";
import Login from "./Login.jsx";
import Vault from "./Vault.jsx";
import CreateItem from "./CreateItem.jsx";

const SECTIONS = Object.freeze({
  home: "HOME",
  vault: "VAULT",
  create: "CREATE",
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
      <>
        <p>Hello</p>
        <Authenticate username={username}/>
        <Login username={username}/>
        <p onClick={() => setSection(SECTIONS.vault)}>Something</p>
      </>
    )
  };

  const renderVault = () => {
    return (
      <Vault
        setSection={setSection}
        sections={SECTIONS}/>
    )
  };

  const renderCreateVaultItem = () => {
    return <CreateItem />;
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
      <div className="container">
        {section === SECTIONS.home && renderHome()}
        {section === SECTIONS.vault && renderVault()}
        {section === SECTIONS.create && renderCreateVaultItem()}
      </div>
    </>
)
}
