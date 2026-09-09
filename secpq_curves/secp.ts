function bigint_to_array(n: number, k: number, x: bigint) {
  let mod: bigint = BigInt(1);
  for (var idx = 0; idx < n; idx++) {
    mod = mod * BigInt(2);
  }

  let ret: bigint[] = [];
  var x_temp: bigint = x;
  for (var idx = 0; idx < k; idx++) {
    ret.push(x_temp % mod);
    x_temp = x_temp / mod;
  }
  return ret;
}

let values = [
  "60197513588986302554485582024885075108884032450952339817679072026166228089408",
  "37718080363155996902926221483475020450927657555482586988616620542887997980018",
];

let output = "";
for (let idx = 0; idx < values.length; idx++) {
  let conv = bigint_to_array(64, 4, BigInt(values[idx]));
  output += "Fq::from_raw([" + conv + "]),\n";
}

console.log(output);
