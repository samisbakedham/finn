# Web wallet

## Prerequisites 

You should already have finn node is up and running. The finn wallet need to be init and ready for usage.

Please check for more details at [finn floonet](https://github.com/cgilliard/finn/blob/master/doc/finn_floonet.md).  

## Web Wallet Security

Please note that web wallet runs in insecure mode. Wallet API will be exposed to all internal processes.
We don't recomment using web wallet.

## Install Web Wallet

finn project has a web wallet which is compartible with finn wallet. It is using Angilar to run it.

### Install Angular CLI

Angular requires Node.js version 8.x or 10.x. To check your version, run `node -v` in a terminal/console window.

To get Node.js, go to [nodejs.org](nodejs.org).

Angular, the Angular CLI, and Angular apps depend on features and functionality 
provided by libraries that are available as [npm packages](https://docs.npmjs.com/getting-started/what-is-npm). 
To download and install npm packages, you must have an npm package manager.

This Quick Start uses the [npm client](https://docs.npmjs.com/cli/install) 
command line interface, which is installed with `Node.js` by default.

To check that you have the npm client installed, run `npm -v` in a terminal/console window.

To install the CLI using npm, open a terminal/console window and enter the following command:
```
npm install -g @angular/cli
```

### Update finn wallet settings

Open the `finn-wallet.toml` and disable owner API authentification by commenting the line:
```text
#api_secret_path = "/Users/kbay/.finn/floo/.api_secret"
```
Change Owner API port to 13420 
```text
owner_api_listen_port = 13420
```

### Start Web Wallet

Get the finn web wallet sources
```bash
> git clone git@github.com:mimblewimble/finn-web-wallet.git
```
Make sure that finn node is running. Start finn wallet in listening mode.
```bash
> cd <wallet_dir>
> finn wallet listen
```
Start web wallet
```bash
> cd finn-web-wallet
> ng build
> ng serve
```
Your web wallet should be available at http://localhost:4200/

