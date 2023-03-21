import { Base64 } from "js-base64";

const getCredentialsBody = async (credentials) => {
  const { challenge, allowCredentials } = credentials.publicKey;

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

  return {
    id: credentialsKeys.id,
    rawId: Base64.fromUint8Array(new Uint8Array(credentialsKeys.rawId), true),
    type: credentialsKeys.type,
    response: {
      authenticatorData: uint8AuthData,
      clientDataJSON: uint8ClientDataJSON,
      signature: uint8Signature,
    },
  };
};

export default getCredentialsBody;