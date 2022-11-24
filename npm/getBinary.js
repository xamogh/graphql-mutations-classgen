const { Binary } = require("binary-install");

function getBinary() {
  var url =
    "https://github.com/xamogh/safe-urqlcodgen-mutations/releases/download/v1.0.0-aplha/safe-urqlcodgen-mutations.tar.gz";
  var name = "safe-urql-codgen-mutations";
  return new Binary(name, url);
}

module.exports = getBinary;
