window.BENCHMARK_DATA = {
  "lastUpdate": 1651713026187,
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
          "id": "506e310d372df5d7b3febfc00fa845f4222beb6c",
          "message": "Merge pull request #570 from zcash/ff-0.12\n\nMigrate to `ff 0.12`",
          "timestamp": "2022-05-05T01:06:48+01:00",
          "tree_id": "c5960c654f9def1a82405087693b6e25c6983dce",
          "url": "https://github.com/zcash/halo2/commit/506e310d372df5d7b3febfc00fa845f4222beb6c"
        },
        "date": 1651713022126,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 75729119,
            "range": "± 2568852",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3405581,
            "range": "± 124592",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 177463505,
            "range": "± 1729532",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4991498,
            "range": "± 209101",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 242596281,
            "range": "± 2405618",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6007819,
            "range": "± 44083",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 36583,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141646,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153835,
            "range": "± 1298",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 249627,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 249485,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144646,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156880,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 252558,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 252576,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 302637,
            "range": "± 1031",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 314886,
            "range": "± 341",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 410511,
            "range": "± 1565",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 410569,
            "range": "± 844",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3173357,
            "range": "± 21617",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5865102,
            "range": "± 6225",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10111370,
            "range": "± 9309",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22173952,
            "range": "± 263901",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 41729370,
            "range": "± 539897",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 82806561,
            "range": "± 329032",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 14624,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 17088,
            "range": "± 645",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 20610,
            "range": "± 228",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 28418,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 45073,
            "range": "± 453",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 80562,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 155904,
            "range": "± 1505",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 314977,
            "range": "± 19321",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 658919,
            "range": "± 18768",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1419893,
            "range": "± 64815",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2992692,
            "range": "± 39442",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 6284034,
            "range": "± 62961",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 13274470,
            "range": "± 347021",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 28448489,
            "range": "± 1221761",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 61051781,
            "range": "± 1026301",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 126351730,
            "range": "± 1766169",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28437,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28535,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 193144177,
            "range": "± 140918",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 408478217,
            "range": "± 415781",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 866817752,
            "range": "± 1656195",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1841556129,
            "range": "± 11446507",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3896571338,
            "range": "± 3006271",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8242873240,
            "range": "± 15773747",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17391267672,
            "range": "± 20402067",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36631462984,
            "range": "± 209894561",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 76957197508,
            "range": "± 204504184",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 96817189,
            "range": "± 527565",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 168635379,
            "range": "± 1442825",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 300309564,
            "range": "± 3136399",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 547173189,
            "range": "± 2265680",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1023053638,
            "range": "± 1748865",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1935692818,
            "range": "± 8354047",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3682939682,
            "range": "± 14127511",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7102134577,
            "range": "± 9757789",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13701340800,
            "range": "± 34968660",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5179712,
            "range": "± 42322",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8032423,
            "range": "± 124318",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12985832,
            "range": "± 96950",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21898564,
            "range": "± 162797",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37206626,
            "range": "± 210420",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66442288,
            "range": "± 447691",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120247939,
            "range": "± 606082",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222278296,
            "range": "± 4148428",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 401422966,
            "range": "± 4012272",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}