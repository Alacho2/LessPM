const performPostRequest = async (url, authToken, body) => {
  const authorized = await fetch(url, {
    method: "POST",
    headers: {
      'Authorization': authToken,
      'Accept': 'application/json',
      'Content-Type': 'application/json',
    },
    credentials: 'include',
    body: JSON.stringify(body),
  });

  return authorized.status;
};

export default performPostRequest;