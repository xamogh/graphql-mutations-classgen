const { Binary } = require("binary-install");

function getBinary() {
  const version = require("../package.json").version;
  const url = `https://github.com/xamogh/safe-urqlcodgen-mutations/releases/download/v1.0.0-aplha/safe-urqlcodgen-mutations.tar.gz`;
  const name = "safe-urql-codgen-mutations";
  return new Binary(url, { name });
}

module.exports = getBinary;
