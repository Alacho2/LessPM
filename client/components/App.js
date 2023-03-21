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
        <p>Hello</p>
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
