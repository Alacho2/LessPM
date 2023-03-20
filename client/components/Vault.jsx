// This is so weird. But we are actually now at a position where we most likely
// have to store passwords in plain text ...
// Otherwise, we'll have to store a hashed/encrypted version of them and use
// Something secret as a key

// The question is whether we can use the passkey ID as a key, and then pad
// it up to 32 bit

import {useEffect, useState} from "react";
import {Base64} from "js-base64";

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
    const fetched_passwords = await fetch(GET_PASSWORDS_URL, {
      method: "GET",
      credentials: "include"
    })

    // User isn't authenticated
    if (fetched_passwords.status !== 200) {
      return;
    }

    const json = await fetched_passwords.json();
    setPasswords(json);
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

    const{ challenge, allowCredentials } = credentials.publicKey;

    credentials.publicKey.challenge = Base64.toUint8Array(challenge);
    credentials.publicKey.allowCredentials = allowCredentials.map(listItem => ({
      ...listItem,
      id: Base64.toUint8Array(listItem.id),
    }));

    const credentialsKeys
        = await navigator.credentials.get({publicKey: credentials.publicKey});

    if (!credentialsKeys) {
      return;
    }

    const {
      authenticatorData,
      clientDataJSON,
      signature,
    } = credentialsKeys.response;

    const uint8AuthData
        = Base64.fromUint8Array(new Uint8Array(authenticatorData));
    const uint8ClientDataJSON
        = Base64.fromUint8Array(new Uint8Array(clientDataJSON));
    const uint8Signature = Base64.fromUint8Array(new Uint8Array(signature));

    const credentialsToSend = {
      id: credentialsKeys.id,
      rawId: Base64.fromUint8Array(new Uint8Array(credentialsKeys.rawId), true),
      type: credentialsKeys.type,
      response: {
        authenticatorData: uint8AuthData,
        clientDataJSON: uint8ClientDataJSON,
        signature: uint8Signature,
      },
    };

    const body = {
      credentials: credentialsToSend,
      objectId: bson,
      process: "retrieval",
    };

    const authorized = await fetch(END_PASSWORD_RETRIEVAL_URL, {
      method: "POST",
      headers: {
        'Authorization': authToken,
        'Accept': "application/json",
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify(body),
    })

    if (authorized !== 200) {
      console.log("Booped out");
      return;
    }

    console.log("Get the password");

    // const password = await fetch(`${GET_PASSWORDS_URL}/${bson}`, {
    //   method: "GET",
    //   credentials: "include",
    // });
    //
    // if (password.status !== 200) {
    //   return;
    // }

    // do a fetch request to another part of the API
  };


  // TODO(Håvard): Create an entry point to check
  // You have to figure out a way to display the creation button
  return (
      <div className="mt-4 bg-light position-relative border rounded vault">
        {true ? <div
            className="create-button position-absolute top-0 end-0"
            onClick={() => props.setSection(props.sections.create)}>
          Create
        </div> : null }
        <div className="mx-3 my-5">
          {passwords.length ? passwords.map((item, i) => {
            const { website, username } = item;

            const url = tryUrlConstruction(website);
            console.log(url);
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

          }) : <p>Doesn't seem like you are authenticated, matey</p>}
        </div>
      </div>
  )
};

export default Vault;
