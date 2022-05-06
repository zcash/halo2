window.BENCHMARK_DATA = {
  "lastUpdate": 1651866643583,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "str4d",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "30f92f3f4b785ea2a32392bf65c1b08f0411567c",
          "message": "Merge pull request #522 from zcash/full-width-var-base-mul\n\n[ECC gadget] Distinguish base field element case in variable-base scalar mul.",
          "timestamp": "2022-05-06T19:53:57+01:00",
          "tree_id": "686b30e3081e044b058851c2cf8cdc60f669e071",
          "url": "https://github.com/zcash/halo2/commit/30f92f3f4b785ea2a32392bf65c1b08f0411567c"
        },
        "date": 1651866638628,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 68922392,
            "range": "± 5110649",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3213253,
            "range": "± 258812",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 160822721,
            "range": "± 5938901",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4752259,
            "range": "± 182815",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 222546267,
            "range": "± 7928573",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5776237,
            "range": "± 246805",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 34271,
            "range": "± 2462",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 131240,
            "range": "± 10976",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 142293,
            "range": "± 8329",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 229534,
            "range": "± 10601",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 231559,
            "range": "± 8106",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135113,
            "range": "± 6377",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 145377,
            "range": "± 5536",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 233169,
            "range": "± 11183",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 233947,
            "range": "± 12045",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 280693,
            "range": "± 17132",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 301116,
            "range": "± 18858",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 378040,
            "range": "± 16309",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 383328,
            "range": "± 18207",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2895841,
            "range": "± 159268",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5532329,
            "range": "± 281439",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 9560187,
            "range": "± 450859",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 20185214,
            "range": "± 383913",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 37164096,
            "range": "± 617631",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 74783446,
            "range": "± 1874328",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7542,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8646,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15959,
            "range": "± 1323",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19828,
            "range": "± 1406",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27333,
            "range": "± 1952",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 45738,
            "range": "± 2938",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 91259,
            "range": "± 7926",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 184563,
            "range": "± 25573",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 383988,
            "range": "± 39497",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 774553,
            "range": "± 65747",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1681506,
            "range": "± 72117",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3731383,
            "range": "± 239144",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8119599,
            "range": "± 331346",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17437069,
            "range": "± 1290933",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 37274990,
            "range": "± 2207579",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 83259348,
            "range": "± 3891875",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 26120,
            "range": "± 1777",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 26139,
            "range": "± 1045",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 151465394,
            "range": "± 26182241",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 315732060,
            "range": "± 9836088",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 680813067,
            "range": "± 27567769",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1444958595,
            "range": "± 18189922",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3139842867,
            "range": "± 79247920",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6650922340,
            "range": "± 82663329",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14304095778,
            "range": "± 119325275",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 30478429301,
            "range": "± 486313993",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 64219358124,
            "range": "± 557599821",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 91238577,
            "range": "± 3420573",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 160091992,
            "range": "± 2120264",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 282177350,
            "range": "± 3896171",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 515116337,
            "range": "± 13421202",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 968927364,
            "range": "± 13868878",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1827514922,
            "range": "± 16967921",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3510713016,
            "range": "± 55033439",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6758329034,
            "range": "± 64597155",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13147703699,
            "range": "± 190187766",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 4966620,
            "range": "± 270807",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7661907,
            "range": "± 390920",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12318837,
            "range": "± 600082",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21069325,
            "range": "± 1343039",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 36105425,
            "range": "± 2842909",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 63325592,
            "range": "± 2843472",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 115493802,
            "range": "± 7330562",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 212207381,
            "range": "± 9980539",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 389414343,
            "range": "± 12685851",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}