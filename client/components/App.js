import {useState} from 'react';
import Authenticate from "./Authenticate.jsx";
import Login from "./Login.jsx";
import Vault from "./Vault.jsx";
import CreateItem from "./CreateItem.jsx";
import checkIsAuthenticated from "./checkIsAuthenticated";

const SECTIONS = Object.freeze({
  home: "HOME",
  vault: "VAULT",
  create: "CREATE",
  login: "LOGIN",
  register: "REGISTER",
});

export function App() {
  const [section, setSection] = useState(SECTIONS.home);
  const [username, setUsername] = useState("");
  const isAuthenticated = checkIsAuthenticated(section);

  const renderHome = () => {
    return (
      <>
        <center>

        <h1>LessPM: The Passwordless Password Manager</h1>
        <p>Tired of remembering dozens of passwords for all your online accounts? LessPM is the solution you've been
          looking for. Our passwordless password manager offers a seamless and secure way to manage your login
          credentials without the need for a master password or complex encryption keys.</p>

        <p>With LessPM, your login credentials are stored locally on your authenticator device using an asymmetric
          encryption scheme. You can quickly access your saved login information with just a few clicks.</p>

          <p>Say goodbye to the hassle of managing passwords and hello to the simplicity and security of LessPM. Try it
            out today and experience the passwordless password manager that's changing the game.</p>
        </center>
      </>
    )
  };

  const renderVault = () => {
    return <Vault
      isAuthenticated={isAuthenticated}
      setSection={setSection}
      sections={SECTIONS}/>

  };

  const renderCreateVaultItem = () => {
    return <CreateItem
      username={username}
      isAuthenticated={isAuthenticated}
      sections={SECTIONS}
      setSection={setSection} />;
  };

  const renderRegister = () => {
    return <Authenticate
      username={username}
      setUsername={setUsername}
      sections={SECTIONS}
      setSection={setSection} />;
  };

  const renderLogin = () => {
    return <Login
      username={username}
      setUsername={setUsername}
      sections={SECTIONS}
      setSection={setSection} />;
  };

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
          <div className="navbar-nav navbar-expand-sm">
            <ul className="navbar-nav ml-auto">
              { isAuthenticated ? <li className="nav-item">
                <span
                  className="nav-link"
                  onClick={() => setSection(SECTIONS.vault)}>
                  Vault
                </span>
              </li> : <>
                <li className="nav-item">
                  <span className="nav-link" onClick={() => setSection(SECTIONS.login)}>
                    Login
                  </span>
                </li>
                <li className="nav-item">
                  <span className="nav-link" onClick={() => setSection(SECTIONS.register)}>
                    Register
                  </span>
                </li>
              </>
              }
            </ul>
          </div>
        </div>
      </div>
      <div className="container">
        {section === SECTIONS.home && renderHome()}
        {section === SECTIONS.vault && renderVault()}
        {section === SECTIONS.create && renderCreateVaultItem()}
        {section === SECTIONS.register && renderRegister()}
        {section === SECTIONS.login && renderLogin()}
      </div>
    </>
  )
}
