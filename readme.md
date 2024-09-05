What is this? 
This is my exam project for Computer Security: Defence Against the Dark Arts at Reykjavik University. It was also my first project in Rust. 
Included is the Latex report so you can do a deep-dive into what was done and what it is.

TL;DR, it is a passwordless password manager that integrates WebAuthn. 

---

Original: 

Dear Prof. XYZ
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
* Rust: v1.67.0
* MongoDB: v6.0.4

I recommend these versions, and you can find install instruction 
for the latest version here: https://www.rust-lang.org/tools/install

Mongo was installed through tapping brew. Install instruction can be found here:
https://www.mongodb.com/docs/manual/tutorial/install-mongodb-on-os-x/

After you have rust and mongodb up and running, you should be able to build the project through 
Before running this command, you need to be in the server folder.
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
javascript you have been using recently. 
You need yarn, as that was the building system that I chose.
After these are installed, you can run these commands from the client-folder.
```
yarn install;
yarn website:https
```
This should fire up the client, which is using https://localhost:1234

Again, if you can't use 1234, you'll have to reconfigure CORS on the server
to the port you want to use. Fun. :-)

If everything goes to plan, checkout https://localhost:1234. 
I tested the system using an iPhone 13 Pro Max, a Samsung S21,
and a Samsung A52.

As you would say,

Have fun!
