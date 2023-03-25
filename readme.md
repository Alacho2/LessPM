Dear Jacky,

There are a few steps to run this project and get it setup.
As the server is running under a self-signed certificate, the first thing 
you have to do after cloning the project is to navigate into 
the server/keys folder. This folder contains a certificate.pem file, 
which you will need to trust. On a mac, it should be nothing more than
double-tapping, add it to the keychain and then accepting it. 
In keychain, double-tap and change the trust values to ALWAYS trust it.

This should permit your Mac to serve the content over the self-signed
certificate, after doing the same thing with the padlock in Chrome, and
changing your site settings for https://localhost:1234. 
This is where the client will run

### Server
The server is developed on an instance of Rust which is v1.67. 
I recommend this version, and you can find install instruction 
for the latest version here: https://www.rust-lang.org/tools/install

After you have rust up and running, you should be able to build the project through 
```
cargo build && cargo run
```
By default, the server runs on port 3000. I figured you probably don't have
anything running on that port. 
If you are using something on that port, shut that down. Given that I was required
to use https (both by Benedikt and for the WebAuthn to work), this is the url the client
communicates with.
If you can't shut down whatever is running on 3000, you'll have to go to the repo and reconfigure CORS, 
change the port in main.rs, and change ALL the URLS client side to your port.
You should find them with a global search of localhost:3000.

Cors lives here:
main.rs, L52-56 
```
    .layer(CorsLayer::new()
      .allow_origin([
        "https://localhost:3000".parse::<HeaderValue>().unwrap(),
        "https://localhost:1234".parse::<HeaderValue>().unwrap()
      ])
      .allow_credentials(true)
```

While the main server port is here:
main.rs, L32-34
```
let ports = Ports {
  https: 3000,
}
```

### Client
Requirements (these were the versions I used): 
* NodeJS: v16.13.1
* Yarn: v3.5.0

The client is, I hope, a little bit simpler. Well, depends how much 
javascript you have been using recently 
You need yarn, as that was the building system that I chose.
After these are installed, you can run
```
yarn install;
yarn website:https
```
This should fire up the client, which is using https://localhost:1234

Again, if you can't use 1234, you'll have to reconfigure CORS on the server
to the port you want to use. Fun. :-)

If everything goes to plan, checkout https://localhost:1234. 


Have fun!