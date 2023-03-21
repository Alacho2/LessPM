// This is so weird. But we are actually now at a position where we most likely
// have to store passwords in plain text ...
// Otherwise, we'll have to store a hashed/encrypted version of them and use
// Something secret as a key

// The question is whether we can use the passkey ID as a key, and then pad
// it up to 32 bit

import {useEffect, useState} from "react";
import getCredentialsBody from "./getCredentialsBody";
import performPostRequest from "./performPostRequest";

// PROPS:
// setSection: (SECTION) => {}
// sections: SECTIONS

const BASE_URL = "https://localhost:3000/";
const GET_PASSWORDS_URL = `${BASE_URL}user/passwords`;
const START_PASSWORD_RETRIEVAL_URL = `${BASE_URL}fido/start_password_creation`;
const END_PASSWORD_RETRIEVAL_URL = `${BASE_URL}fido/end_password_creation`;
const FALLBACK_URL = "https://ru.is";
const AUTH_HEADER = "authorization";

const Vault = (props) => {
  const [passwords, setPasswords] = useState([]);
  const [passwordMap, setPasswordMap] = useState(new Map());

  useEffect(() => {
    getPasswords();
  }, []);

  const getPasswords = async () => {
    try {

      const fetchedPasswords = await fetch(GET_PASSWORDS_URL, {
        method: "GET",
        credentials: "include"
      });

      // User isn't authenticated
      if (fetchedPasswords.status !== 200) {
        return;
      }

      const json = await fetchedPasswords.json();
      setPasswords(json);
    } catch { /* Don't do anything */ }
  };
  const handleImageError = (target) => {
    target.currentTarget.onerror = null;
    target.currentTarget.style = "background: black";
    target.currentTarget.src = "https://www.ru.is/skin/basic9k/i/foot-logo-mobile.png";
  };

  // PARAMS:
  // website: string
  const tryUrlConstruction = (website) => {
    try {
      return new URL(website);
    } catch {
      return new URL(FALLBACK_URL);
    }
  };

  // StrippedVaultItem
  const getBson = (item) => {
    return item["_id"]["$oid"];
  };

  // StrippedVaultItem
  const getOnePassword = async (item) => {
    try {
      const bson = getBson(item);

      const startPasswordRetrieval = await fetch(START_PASSWORD_RETRIEVAL_URL, {
        method: "POST",
        credentials: "include",
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      });

      if (startPasswordRetrieval.status !== 200) {
        console.log("BOOOOO");
        return; // exit early
      }

      const credentials = await startPasswordRetrieval.json();
      const authToken = startPasswordRetrieval.headers.get(AUTH_HEADER);

      const credentialsToSend = await getCredentialsBody(credentials);

      if (!credentialsToSend) {
        return;
      }

      const body = {
        credentials: credentialsToSend,
        objectId: bson,
        process: "retrieval",
      };

      const {status, rtnBody} = await performPostRequest(END_PASSWORD_RETRIEVAL_URL, authToken, body);

      if (status !== 200) {
        console.log("Booped out");
        return;
      }

      // get the password and schedule a timer to remove it in 15 seconds.
      setPasswordMap(prevState => {
        setTimeout(() => {
          setPasswordMap(innerPrevState => {
            const cloned = new Map(innerPrevState);
            cloned.delete(bson);
            return cloned;
          });
        }, 15000);
        return new Map(prevState).set(bson, rtnBody.msg);
      })

    } catch { /* Don't do anything */ }
  };

  const isAuthenticated = props.isAuthenticated;

  return (
    <div className="mt-4 bg-light position-relative border rounded vault">
      {isAuthenticated &&
        <div
          className="create-button position-absolute top-0 end-0"
          onClick={() => props.setSection(props.sections.create)}>
          Create
        </div>
      }
      <div className="mx-3 my-5">
        <div className="container">
          {isAuthenticated && passwords.length > 0
            && passwords.map((item, i) => {
              const { website, username } = item;
              const bson = getBson(item);
              const doesBsonExist = passwordMap.get(bson);


              const url = tryUrlConstruction(website);
              return (
                <div
                  className="row border"
                  key={i}>
                  <div
                    className="col-6 item d-flex flex-row"
                    onClick={() => getOnePassword(item)}>
                    <img
                      className="favicon"
                      src={`${url.origin}/favicon.ico`}
                      onError={handleImageError}/>
                    <div>
                      <p className="website">{website}</p>
                      <p className="username">{username}</p>
                    </div>
                  </div>
                  <div className="col-6 item d-flex flex-row">{
                    doesBsonExist && <div className="input-group">
                      <input
                        type="text" // This depends on the checkbox
                        className="form-control"
                        value={doesBsonExist}
                        onChange={({target}) => setPassword(target.value) }
                        disabled
                        aria-label="Text input with checkbox" />
                      <div
                        onClick={() => navigator.clipboard.writeText(doesBsonExist)}
                        className="input-group-append clipboard">
                        <span className="input-group-text">
                          <i className="fas fa-clipboard"></i>
                        </span>
                      </div>
                    </div>
                  }</div>
                </div>
              );

            })
          }
        </div>

      {isAuthenticated && !passwords.length && <p>Doesn't seem like you have any passwords, matey</p>}
      {!isAuthenticated && <p>Not sure how you ended up here but yeah...</p>}
      </div>
    </div>
  )
};

export default Vault;
