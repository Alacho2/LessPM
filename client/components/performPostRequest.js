
const performPostRequest = (url, authToken, body) => {
  const authorized = async () => {
    const result = await fetch(url, {
      method: "POST",
      headers: {
        'Authorization': authToken,
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify(body),
    });


    let rspBody = "";
    try {
      rspBody = await result.json();
    } catch { /* Don't do anything. Some bodies are just different */}

    return {status: result.status, rtnBody: rspBody}
  };

  return authorized();
};

export default performPostRequest;