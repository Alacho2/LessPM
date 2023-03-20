// This is so weird. But we are actually now at a position where we most likely
// have to store passwords in plain text ...
// Otherwise, we'll have to store a hashed/encrypted version of them and use
// Something secret as a key

// The question is whether we can use the passkey ID as a key, and then pad
// it up to 32 bit

import {useEffect, useState} from "react";
import {Base64} from "js-base64";
import checkIsAuthenticated from "./checkIsAuthenticated";
import getCredentialsBody from "./getCredentialsBody";

const dummy_passwords = [
  {
    id: Math.random().toString(36).substring(2),
    website: "https://google.com",
    username: "havard@alacho.no",
    favicon: "https://google.com/favicon.ico" // when this comes in, you need to strip the path off of the domain
    // favicon needs an onerror which handles domains that doesn't exist and sets
    // the source to the default bullshit image
  }, {
    id: Math.random().toString(36).substring(2),
    website: "https://stackoverflow.com",
    username: "havard@alacho.no",
    favicon: "https://stackoverflow.com/favicon.ico" // when this comes in, you need to strip the path off of the domain
    // favicon needs an onerror which handles domains that doesn't exist and sets
    // the source to the default bullshit image
  }, {
    id: Math.random().toString(36).substring(2),
    website: "https://facebook.com",
    username: "havard@alacho.no",
    favicon: "https://facebook.com/favicon.ico" // when this comes in, you need to strip the path off of the domain
    // favicon needs an onerror which handles domains that doesn't exist and sets
    // the source to the default bullshit image
  }, {
    id: Math.random().toString(36).substring(2),
    website: "https://twitter.com",
    username: "alacho_",
    favicon: "https://twitter.com/favicon.ico" // when this comes in, you need to strip the path off of the domain
    // favicon needs an onerror which handles domains that doesn't exist and sets
    // the source to the default bullshit image
  }, {
    id: Math.random().toString(36).substring(2),
    website: "https://discord.com",
    username: "Alacho",
    favicon: "https://discordapp.com/favicon.ico" // when this comes in, you need to strip the path off of the domain
    // favicon needs an onerror which handles domains that doesn't exist and sets
    // the source to the default bullshit image
  }, {
    id: Math.random().toString(36).substring(2),
    website: "https://vg.no",
    username: "havard@alacho.no",
    favicon: "https://vg.no/favicon.ico" // when this comes in, you need to strip the path off of the domain
    // favicon needs an onerror which handles domains that doesn't exist and sets
    // the source to the default bullshit image
  },
];

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
  const getOnePassword = async (item) => {
    try {

      const bson = item["_id"]["$oid"];

      const user_name = "bjoggii";

      const startPasswordRetrieval = await fetch(START_PASSWORD_RETRIEVAL_URL, {
        method: "POST",
        credentials: "include",
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({name: user_name})
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

      const authorized = await performPostRequest(END_PASSWORD_RETRIEVAL_URL, authToken, body);

      if (authorized !== 200) {
        console.log("Booped out");
        return;
      }
    } catch { /* Don't do anything */ }
  };

  const isAuthenticated = checkIsAuthenticated();

  // TODO(Håvard): Create an entry point to check
  // You have to figure out a way to display the creation button
  return (
    <div className="mt-4 bg-light position-relative border rounded vault">
      {isAuthenticated ? <div
        className="create-button position-absolute top-0 end-0"
        onClick={() => props.setSection(props.sections.create)}>
        Create
      </div> : null }
      <div className="mx-3 my-5">
        {isAuthenticated && passwords.length
          ? passwords.map((item, i) => {
            const { website, username } = item;

            const url = tryUrlConstruction(website);
            return (
              <div
                key={i}
                className="item d-flex flex-row border"
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
            );

          }) : isAuthenticated
            ? <p>Doesn't seem like you are authenticated, matey</p>
            : <p>Not signed in.</p>}
      </div>
    </div>
  )
};

export default Vault;
