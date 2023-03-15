// This is so weird. But we are actually now at a position where we most likely
// have to store passwords in plain text ...
// Otherwise, we'll have to store a hashed/encrypted version of them and use
// Something secret as a key

// The question is whether we can use the passkey ID as a key, and then pad
// it up to 32 bit

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

const Vault = (props) => {

  const handleImageError = (target) => {
    console.log("Fired", target);
    target.currentTarget.onerror = null;
    target.currentTarget.style = "background: black";
    target.currentTarget.src = "https://www.ru.is/skin/basic9k/i/foot-logo-mobile.png";
  };

  return (
    <div className="mt-4 bg-light position-relative border rounded vault">
      <div
        className="create-button position-absolute top-0 end-0"
        onClick={() => props.setSection(props.sections.create)}>
        Create
      </div>
      <div className="mx-3 my-5">
        {dummy_passwords.map((item, i) => {
          const { website, favicon, username } = item;
          return (
            <div
              key={i}
              className="item d-flex flex-row border"
              onClick={() => console.log("server request for auth")}>
              <img
                className="favicon"
                src={favicon}
                onError={handleImageError}/>
              <div>
              <p className="website">{website}</p>
              <p className="username">{username}</p>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  )
};

export default Vault;
