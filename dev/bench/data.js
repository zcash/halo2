window.BENCHMARK_DATA = {
  "lastUpdate": 1764868444691,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira-Emma Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7df93fd855395dcdb301a857d4b33f37903bbf76",
          "message": "Merge pull request #814 from adria0/fix/mdbook\n\nFix MD book generation",
          "timestamp": "2024-02-26T23:50:17Z",
          "tree_id": "ef67992c62cbe95d9ecf5d0fada00c9835333a5b",
          "url": "https://github.com/zcash/halo2/commit/7df93fd855395dcdb301a857d4b33f37903bbf76"
        },
        "date": 1708994300990,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71672766,
            "range": "± 5797578",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4063888,
            "range": "± 56115",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136042062,
            "range": "± 2919441",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4609210,
            "range": "± 154712",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 187547850,
            "range": "± 1100421",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5033443,
            "range": "± 100876",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31228,
            "range": "± 1182",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 133293,
            "range": "± 534",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 146463,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 232196,
            "range": "± 890",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 232363,
            "range": "± 293",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 136061,
            "range": "± 533",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 149290,
            "range": "± 265",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 234880,
            "range": "± 482",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 235069,
            "range": "± 907",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 284744,
            "range": "± 629",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 298350,
            "range": "± 547",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 383589,
            "range": "± 1528",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 383560,
            "range": "± 611",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2892533,
            "range": "± 14944",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 4691817,
            "range": "± 27578",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 8127576,
            "range": "± 80071",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 18492745,
            "range": "± 244348",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 32281226,
            "range": "± 44652",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 62301891,
            "range": "± 108958",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8761,
            "range": "± 2128",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8429,
            "range": "± 1274",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14145,
            "range": "± 993",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18396,
            "range": "± 1362",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26835,
            "range": "± 1316",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 44484,
            "range": "± 2258",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 91253,
            "range": "± 1568",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 162906,
            "range": "± 1820",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 296817,
            "range": "± 3658",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 577470,
            "range": "± 10183",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1154750,
            "range": "± 20912",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 2385698,
            "range": "± 44748",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 5048670,
            "range": "± 169072",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 10920752,
            "range": "± 151924",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 22998723,
            "range": "± 347200",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 54117307,
            "range": "± 1193842",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 29200,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 29301,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 130190318,
            "range": "± 3179730",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 268863406,
            "range": "± 6269136",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 586482216,
            "range": "± 17261695",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1233152223,
            "range": "± 31935882",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 2577197051,
            "range": "± 58126526",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 5455174441,
            "range": "± 126163343",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 11611496728,
            "range": "± 219230400",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 24132876358,
            "range": "± 484489667",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 50543795891,
            "range": "± 1078173075",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 84856217,
            "range": "± 622874",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 139818098,
            "range": "± 1043650",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 237754952,
            "range": "± 1381059",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 420325727,
            "range": "± 7762614",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 761101094,
            "range": "± 3759679",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1415358473,
            "range": "± 4352126",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 2667055509,
            "range": "± 6199187",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 5061309687,
            "range": "± 25088881",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 9781532497,
            "range": "± 23366617",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 4564046,
            "range": "± 136953",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 6651696,
            "range": "± 257929",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 10643673,
            "range": "± 334091",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 17439924,
            "range": "± 514831",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 30006298,
            "range": "± 362072",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 51765311,
            "range": "± 535794",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 92563733,
            "range": "± 843845",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 166515530,
            "range": "± 1066634",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 309772286,
            "range": "± 6945622",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "94cf956afcf090c931b6fbd223ededd6e7588511",
          "message": "Merge pull request #830 from zcash/halo2_poseidon-empty-crate\n\nhalo2_poseidon: Initial empty library crate",
          "timestamp": "2024-12-14T12:56:17+13:00",
          "tree_id": "f095bfa58746081160568f7cf5d27f219e95c5b3",
          "url": "https://github.com/zcash/halo2/commit/94cf956afcf090c931b6fbd223ededd6e7588511"
        },
        "date": 1734134663043,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71354997,
            "range": "± 5012873",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4005742,
            "range": "± 191941",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136227388,
            "range": "± 3211773",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4553323,
            "range": "± 136876",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 187671836,
            "range": "± 4797694",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5047599,
            "range": "± 99596",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31195,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 133154,
            "range": "± 916",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 146321,
            "range": "± 564",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 235873,
            "range": "± 6170",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 235738,
            "range": "± 3297",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135838,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 149036,
            "range": "± 1177",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 238494,
            "range": "± 720",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 238443,
            "range": "± 1020",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 284363,
            "range": "± 897",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 297920,
            "range": "± 1104",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 387069,
            "range": "± 1542",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 387017,
            "range": "± 1090",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7a39f55246dda5cf78e6c849db3830b524402a19",
          "message": "Merge pull request #829 from zcash/sinsemilla-primitive\n\nMove Sinsemilla primitive impl into `sinsemilla` crate",
          "timestamp": "2024-12-14T23:21:30+13:00",
          "tree_id": "eeb19d67e8b820957ba48105ceea2eabf2176c1a",
          "url": "https://github.com/zcash/halo2/commit/7a39f55246dda5cf78e6c849db3830b524402a19"
        },
        "date": 1734172178312,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71674134,
            "range": "± 5016933",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4094539,
            "range": "± 190814",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 135912472,
            "range": "± 2655909",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4553306,
            "range": "± 155314",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 187525144,
            "range": "± 4277508",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4986578,
            "range": "± 108572",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31467,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132692,
            "range": "± 1116",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145927,
            "range": "± 622",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 234001,
            "range": "± 788",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 233851,
            "range": "± 1249",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135356,
            "range": "± 2696",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148667,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 236760,
            "range": "± 417",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 236696,
            "range": "± 1488",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 283358,
            "range": "± 1267",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 297488,
            "range": "± 1230",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 384234,
            "range": "± 979",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 384669,
            "range": "± 1144",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0d1851b57a273cb0b6260a1f4f1168d00b3321ca",
          "message": "Merge pull request #833 from zcash/fix-ci\n\nCI fixes",
          "timestamp": "2024-12-17T04:30:40+13:00",
          "tree_id": "2db3bce48e6ecb0c6ebb4694e05935c10100ea1f",
          "url": "https://github.com/zcash/halo2/commit/0d1851b57a273cb0b6260a1f4f1168d00b3321ca"
        },
        "date": 1734363523917,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71301201,
            "range": "± 5596835",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4044846,
            "range": "± 178294",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 135360523,
            "range": "± 3561703",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4584518,
            "range": "± 156087",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 186674644,
            "range": "± 5945683",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5008118,
            "range": "± 109112",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31142,
            "range": "± 856",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132422,
            "range": "± 641",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145211,
            "range": "± 653",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 234227,
            "range": "± 4953",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 234188,
            "range": "± 1750",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135142,
            "range": "± 3442",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 147824,
            "range": "± 733",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 236958,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 237033,
            "range": "± 933",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 282876,
            "range": "± 2021",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 295323,
            "range": "± 863",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 384581,
            "range": "± 1334",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 384509,
            "range": "± 1335",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c91cc5cd1bf1ca48cef2fea33eb50f75d5420378",
          "message": "Merge pull request #831 from zcash/poseidon-primitive\n\nMove Poseidon primitive into `halo2_poseidon`",
          "timestamp": "2024-12-17T05:21:36+13:00",
          "tree_id": "1cc04ee701239a2ec7f18cc96111d59a24993584",
          "url": "https://github.com/zcash/halo2/commit/c91cc5cd1bf1ca48cef2fea33eb50f75d5420378"
        },
        "date": 1734366588579,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72291251,
            "range": "± 2729921",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4034735,
            "range": "± 59292",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 137162014,
            "range": "± 3307093",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4612882,
            "range": "± 143098",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 189350152,
            "range": "± 3722679",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5026296,
            "range": "± 160330",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31315,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132283,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 144858,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 233436,
            "range": "± 2301",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 233362,
            "range": "± 15897",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 134986,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 147580,
            "range": "± 335",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 236040,
            "range": "± 807",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 236040,
            "range": "± 1282",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 282381,
            "range": "± 17410",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 295001,
            "range": "± 1056",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 383534,
            "range": "± 13586",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 383561,
            "range": "± 3338",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e374954629b65ae21ee37dbaa3159a1ff36d51c0",
          "message": "Merge pull request #834 from zcash/poseidon-no-std\n\nhalo2_poseidon: Add no-std support",
          "timestamp": "2024-12-17T06:51:34+13:00",
          "tree_id": "68666757118b037c223680e0505d4686e88ab556",
          "url": "https://github.com/zcash/halo2/commit/e374954629b65ae21ee37dbaa3159a1ff36d51c0"
        },
        "date": 1734371971845,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71231190,
            "range": "± 1277771",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4037362,
            "range": "± 93297",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 135317434,
            "range": "± 3446179",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4575593,
            "range": "± 147373",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 187824714,
            "range": "± 3559462",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4997371,
            "range": "± 152401",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31257,
            "range": "± 619",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 128959,
            "range": "± 3040",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 144341,
            "range": "± 3027",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 229834,
            "range": "± 5692",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 230625,
            "range": "± 5318",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 134591,
            "range": "± 2760",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 144387,
            "range": "± 3681",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 233250,
            "range": "± 5165",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 235306,
            "range": "± 4548",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 281722,
            "range": "± 5464",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 292584,
            "range": "± 6086",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 373929,
            "range": "± 10509",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 379268,
            "range": "± 8459",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fed6b000857f27e23ddb07454da8bde4697204f7",
          "message": "Merge pull request #835 from zcash/halo2_poseidon-0.1.0\n\nhalo2_poseidon 0.1.0",
          "timestamp": "2024-12-17T08:02:56+13:00",
          "tree_id": "052371a115a6f3a4d91bcbdf00fa4ee5680d4ee7",
          "url": "https://github.com/zcash/halo2/commit/fed6b000857f27e23ddb07454da8bde4697204f7"
        },
        "date": 1734376270167,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71938818,
            "range": "± 5178560",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4020442,
            "range": "± 78840",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 137220690,
            "range": "± 3734548",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4615764,
            "range": "± 237858",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 188938919,
            "range": "± 4270468",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5002683,
            "range": "± 93895",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31105,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132873,
            "range": "± 3612",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145941,
            "range": "± 441",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 236910,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 238055,
            "range": "± 901",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135654,
            "range": "± 408",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148595,
            "range": "± 672",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 239610,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 239685,
            "range": "± 653",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 283822,
            "range": "± 1166",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 297026,
            "range": "± 909",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 389212,
            "range": "± 2380",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 388985,
            "range": "± 2120",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3b4f2cb792bddf795d706401285476f8c6b981a3",
          "message": "Merge pull request #823 from QED-it/improve-backward-compatability-without-zsa\n\nAdd a LookupRangeCheck trait and minor modifications in preparation for ZSA",
          "timestamp": "2025-06-12T13:28:42+01:00",
          "tree_id": "5d67b3d16fcf2a97e3c25cb3b0086ef41f153378",
          "url": "https://github.com/zcash/halo2/commit/3b4f2cb792bddf795d706401285476f8c6b981a3"
        },
        "date": 1749731948495,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72022816,
            "range": "± 5744208",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4075556,
            "range": "± 27473",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 137489806,
            "range": "± 2628944",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4613225,
            "range": "± 83629",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 188961712,
            "range": "± 730191",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5022917,
            "range": "± 29414",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 29942,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 133460,
            "range": "± 658",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 146448,
            "range": "± 653",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 240420,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 240370,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 136423,
            "range": "± 2435",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 149168,
            "range": "± 473",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 243060,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 243094,
            "range": "± 564",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 285204,
            "range": "± 520",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 298064,
            "range": "± 2667",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 391966,
            "range": "± 2243",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 392246,
            "range": "± 2258",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8056703404299dd0a1e381ecfaa780f891dfc392",
          "message": "Merge pull request #837 from wowinter13/feat/improve_mock_provider_docs\n\ndocs(halo2_proofs): Improve MockProver::verify description",
          "timestamp": "2025-06-12T13:30:17+01:00",
          "tree_id": "f0a93f27e83d1cabb9886358e96935f4aeeb8f59",
          "url": "https://github.com/zcash/halo2/commit/8056703404299dd0a1e381ecfaa780f891dfc392"
        },
        "date": 1749732037110,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72018572,
            "range": "± 646524",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4050215,
            "range": "± 42768",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 137725720,
            "range": "± 2533194",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4647473,
            "range": "± 81053",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 189825302,
            "range": "± 750631",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5036867,
            "range": "± 45709",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 29945,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 133347,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 147269,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 235624,
            "range": "± 1874",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 235555,
            "range": "± 736",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135980,
            "range": "± 424",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 149922,
            "range": "± 542",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 238297,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 238355,
            "range": "± 837",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 284873,
            "range": "± 780",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 299707,
            "range": "± 748",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 387170,
            "range": "± 1998",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 387189,
            "range": "± 10252",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5741cc56562cb09b66bd08945b90d45c170cd09b",
          "message": "Merge pull request #846 from zcash/bugfix\n\nReject two queries with the same point and commitment, but different evaluations",
          "timestamp": "2025-07-09T13:32:25+01:00",
          "tree_id": "1ba358fde11937c0ab06f24a406d714dcac5de38",
          "url": "https://github.com/zcash/halo2/commit/5741cc56562cb09b66bd08945b90d45c170cd09b"
        },
        "date": 1752064953917,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72578406,
            "range": "± 1045914",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4120055,
            "range": "± 32036",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 138093434,
            "range": "± 4619444",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4628725,
            "range": "± 97055",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 189768148,
            "range": "± 1437376",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5044170,
            "range": "± 58880",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31684,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 133019,
            "range": "± 4438",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145735,
            "range": "± 5876",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 235813,
            "range": "± 731",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 235969,
            "range": "± 543",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135938,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148536,
            "range": "± 3451",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 238628,
            "range": "± 10043",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 238556,
            "range": "± 6011",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 284284,
            "range": "± 743",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 296630,
            "range": "± 7452",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 387139,
            "range": "± 972",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 386818,
            "range": "± 1238",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira-Emma Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2308caf68c48c02468b66cfc452dad54e355e32f",
          "message": "Merge pull request #845 from QED-it/zsa1\n\nAdd ZSA features",
          "timestamp": "2025-08-06T12:36:01+01:00",
          "tree_id": "4319462c0713713414c3160b954cb6dda72f0f0c",
          "url": "https://github.com/zcash/halo2/commit/2308caf68c48c02468b66cfc452dad54e355e32f"
        },
        "date": 1754480921281,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72199177,
            "range": "± 4791382",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4048931,
            "range": "± 38167",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 137786132,
            "range": "± 2608497",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4672726,
            "range": "± 99106",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 189561673,
            "range": "± 921551",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5116990,
            "range": "± 74681",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31377,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132819,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145947,
            "range": "± 2131",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 234427,
            "range": "± 677",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 234503,
            "range": "± 3522",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135638,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148636,
            "range": "± 1898",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 237312,
            "range": "± 720",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 237252,
            "range": "± 660",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 283638,
            "range": "± 741",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 297174,
            "range": "± 819",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 385279,
            "range": "± 787",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 385426,
            "range": "± 7159",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "afd29b108f6349644d108fab79e3289e1a56e80b",
          "message": "Merge pull request #817 from oxarbitrage/patch-1\n\nFix links from book",
          "timestamp": "2025-12-02T13:46:00Z",
          "tree_id": "76835e599248573e9cfc3f8c4658111e5c0eeed4",
          "url": "https://github.com/zcash/halo2/commit/afd29b108f6349644d108fab79e3289e1a56e80b"
        },
        "date": 1764683729879,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72507950,
            "range": "± 647324",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4137100,
            "range": "± 37053",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 137756942,
            "range": "± 3565237",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4610352,
            "range": "± 110333",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 189606849,
            "range": "± 815137",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5121331,
            "range": "± 79777",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31672,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 133117,
            "range": "± 662",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145738,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 234079,
            "range": "± 1005",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 234132,
            "range": "± 1452",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135865,
            "range": "± 254",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148489,
            "range": "± 327",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 236801,
            "range": "± 466",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 236815,
            "range": "± 1102",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 284623,
            "range": "± 742",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 296571,
            "range": "± 622",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 385408,
            "range": "± 767",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 385246,
            "range": "± 990",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7d5ddecccdc31b11bdad44b5a93b329ade1cbe4b",
          "message": "Merge pull request #855 from Dedme/expose-commitment-k\n\nAdd method to retrieve circuit size parameter k",
          "timestamp": "2025-12-02T14:46:04Z",
          "tree_id": "5e11871092ee4aa34588f27f333dbc6f2715f07e",
          "url": "https://github.com/zcash/halo2/commit/7d5ddecccdc31b11bdad44b5a93b329ade1cbe4b"
        },
        "date": 1764687322204,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71972048,
            "range": "± 614008",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4102729,
            "range": "± 40466",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136774976,
            "range": "± 3471464",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4661876,
            "range": "± 88880",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 188385148,
            "range": "± 870780",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4977814,
            "range": "± 53928",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31389,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132845,
            "range": "± 13983",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145709,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 236185,
            "range": "± 711",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 236088,
            "range": "± 1184",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135557,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148537,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 238883,
            "range": "± 502",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 238912,
            "range": "± 430",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 283883,
            "range": "± 665",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 296690,
            "range": "± 717",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 387225,
            "range": "± 612",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 387211,
            "range": "± 830",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d10fc24f8aa3c42e22d48e115d6bc0909e0c917f",
          "message": "Merge pull request #856 from zcash/release-prep\n\nRelease preparation",
          "timestamp": "2025-12-02T20:29:07Z",
          "tree_id": "e89e8d62250175262f828bd515966299c53186e4",
          "url": "https://github.com/zcash/halo2/commit/d10fc24f8aa3c42e22d48e115d6bc0909e0c917f"
        },
        "date": 1764707915810,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72164580,
            "range": "± 651026",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4093824,
            "range": "± 40269",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 137527151,
            "range": "± 2568826",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4656290,
            "range": "± 93688",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 188993014,
            "range": "± 933235",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5003639,
            "range": "± 43166",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31379,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132907,
            "range": "± 450",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145823,
            "range": "± 254",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 234593,
            "range": "± 3990",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 234575,
            "range": "± 1170",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135607,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148335,
            "range": "± 332",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 237253,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 237165,
            "range": "± 399",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 284184,
            "range": "± 4509",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 296662,
            "range": "± 1343",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 385471,
            "range": "± 655",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 385451,
            "range": "± 721",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3db2f6865955374cb9408739929d60ff412a3bc4",
          "message": "Merge pull request #812 from DamianStraszak/constant-in-simple-example\n\nMake docs consistent with code example.",
          "timestamp": "2025-12-03T19:04:29Z",
          "tree_id": "3edf31d6039aa426c78b3fd8a9803492be9068d3",
          "url": "https://github.com/zcash/halo2/commit/3db2f6865955374cb9408739929d60ff412a3bc4"
        },
        "date": 1764789252222,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72314920,
            "range": "± 586181",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4086514,
            "range": "± 35783",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 137975998,
            "range": "± 2982819",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4644103,
            "range": "± 92051",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 189709487,
            "range": "± 879237",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5045626,
            "range": "± 74058",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31404,
            "range": "± 378",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132960,
            "range": "± 236",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 146121,
            "range": "± 325",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 237106,
            "range": "± 515",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 237030,
            "range": "± 413",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135734,
            "range": "± 959",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148839,
            "range": "± 304",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 239669,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 239600,
            "range": "± 471",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 284025,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 297201,
            "range": "± 611",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 387815,
            "range": "± 1041",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 388202,
            "range": "± 675",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "aac9dcaaa7fdd8b3aa28fb8c1b24f5ae1157111a",
          "message": "Merge pull request #793 from zhiqiangxu/opt_build_vk\n\nminor optimize `build_vk`: avoid realloc",
          "timestamp": "2025-12-03T19:07:36Z",
          "tree_id": "160ebc34975207b72644a880d50ca6af22ff48fb",
          "url": "https://github.com/zcash/halo2/commit/aac9dcaaa7fdd8b3aa28fb8c1b24f5ae1157111a"
        },
        "date": 1764789423222,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72005243,
            "range": "± 6164047",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4119667,
            "range": "± 43217",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 137217037,
            "range": "± 2956641",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4607339,
            "range": "± 102155",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 189255313,
            "range": "± 726701",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5040857,
            "range": "± 44541",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31691,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132679,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145776,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 234703,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 234710,
            "range": "± 768",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135507,
            "range": "± 573",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148599,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 237438,
            "range": "± 2992",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 237388,
            "range": "± 425",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 283675,
            "range": "± 587",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 296912,
            "range": "± 490",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 385706,
            "range": "± 744",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 385545,
            "range": "± 1081",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dc2b61d9e723c83fb9b7c47c2392309154d10d2c",
          "message": "Merge pull request #791 from zhiqiangxu/ensure_extended_k_le_s\n\nensure extended_k <= S",
          "timestamp": "2025-12-03T19:09:25Z",
          "tree_id": "75de8cc6562444cd8c109c5401512742b6f76617",
          "url": "https://github.com/zcash/halo2/commit/dc2b61d9e723c83fb9b7c47c2392309154d10d2c"
        },
        "date": 1764789538396,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72317430,
            "range": "± 6151237",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4039563,
            "range": "± 38348",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 138277225,
            "range": "± 2762530",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4665397,
            "range": "± 108553",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 189763539,
            "range": "± 828771",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5010132,
            "range": "± 49335",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31365,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132770,
            "range": "± 408",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145876,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 236031,
            "range": "± 7356",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 236005,
            "range": "± 596",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135449,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148763,
            "range": "± 319",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 238622,
            "range": "± 777",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 238753,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 283905,
            "range": "± 2847",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 297297,
            "range": "± 1075",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 386937,
            "range": "± 732",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 386887,
            "range": "± 733",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "509ca41a006a9f65d5a3f08f193e87c0afcca1d9",
          "message": "Merge pull request #792 from zhiqiangxu/opt_usage_pow_vartime\n\n`pow_vartime([n, 0, 0, 0]` => `pow_vartime([n]`",
          "timestamp": "2025-12-03T19:18:18Z",
          "tree_id": "b0afbb8b25f234288c0510653dc399d519681ad2",
          "url": "https://github.com/zcash/halo2/commit/509ca41a006a9f65d5a3f08f193e87c0afcca1d9"
        },
        "date": 1764790073365,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71976299,
            "range": "± 1706101",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4069907,
            "range": "± 37457",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136453218,
            "range": "± 2654856",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4650492,
            "range": "± 103745",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 188134199,
            "range": "± 2048102",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5064254,
            "range": "± 53911",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31396,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 133285,
            "range": "± 372",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 146029,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 233779,
            "range": "± 498",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 233721,
            "range": "± 598",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 136232,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148791,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 236498,
            "range": "± 677",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 236396,
            "range": "± 449",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 285097,
            "range": "± 2285",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 297148,
            "range": "± 1152",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 384612,
            "range": "± 754",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 384835,
            "range": "± 742",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "kris@nutty.land",
            "name": "Kris Nuttycombe",
            "username": "nuttycom"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2d61063abff7fa7510762fa66e9be48d67b7ea76",
          "message": "Merge pull request #857 from zcash/ci-fixes\n\nCI fixes",
          "timestamp": "2025-12-03T16:31:30-07:00",
          "tree_id": "17145937f741fedd6c48295f9db61178336f4af5",
          "url": "https://github.com/zcash/halo2/commit/2d61063abff7fa7510762fa66e9be48d67b7ea76"
        },
        "date": 1764805267354,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71853978,
            "range": "± 596283",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4124160,
            "range": "± 48444",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136752169,
            "range": "± 3212393",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4708326,
            "range": "± 88625",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 188102222,
            "range": "± 1195081",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5029314,
            "range": "± 44201",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31688,
            "range": "± 317",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 133271,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 146381,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 236219,
            "range": "± 503",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 236454,
            "range": "± 755",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 136087,
            "range": "± 201",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 149161,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 239070,
            "range": "± 823",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 239016,
            "range": "± 638",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 285014,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 298162,
            "range": "± 1170",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 387006,
            "range": "± 2087",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 387374,
            "range": "± 1136",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "bf462fa2403283b2bbe6db98ada07dd3fd67bd0c",
          "message": "Merge pull request #858 from zcash/ci-fixes-2\n\nCI: Migrate to `actions/upload-artifact@v5`",
          "timestamp": "2025-12-04T02:32:56Z",
          "tree_id": "6859e96ce42a62abecd307cc3bb86234d39e8d47",
          "url": "https://github.com/zcash/halo2/commit/bf462fa2403283b2bbe6db98ada07dd3fd67bd0c"
        },
        "date": 1764816148173,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72089746,
            "range": "± 623795",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4056869,
            "range": "± 41129",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136797425,
            "range": "± 5179241",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4643489,
            "range": "± 94916",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 188318987,
            "range": "± 936503",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5045907,
            "range": "± 40854",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31698,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132871,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145799,
            "range": "± 286",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 236543,
            "range": "± 772",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 236657,
            "range": "± 545",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135521,
            "range": "± 276",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148529,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 239410,
            "range": "± 959",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 239279,
            "range": "± 1968",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 283725,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 296864,
            "range": "± 1560",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 387612,
            "range": "± 6417",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 387516,
            "range": "± 3789",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c7da55009ade3909a9aa5fa93a7ca78e3dfaa90d",
          "message": "Merge pull request #859 from zcash/book-fixes\n\nCI: Build book with `mdbook 0.4`",
          "timestamp": "2025-12-04T02:52:20Z",
          "tree_id": "f224772527e88a5ecd7079b64e2e8d8f87224260",
          "url": "https://github.com/zcash/halo2/commit/c7da55009ade3909a9aa5fa93a7ca78e3dfaa90d"
        },
        "date": 1764817319245,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71769566,
            "range": "± 756642",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4086092,
            "range": "± 39766",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136524680,
            "range": "± 2141013",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4446742,
            "range": "± 50832",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 186088488,
            "range": "± 1015983",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5039592,
            "range": "± 58372",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 28999,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 114673,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 125549,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 204930,
            "range": "± 673",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 204992,
            "range": "± 694",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 117082,
            "range": "± 387",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 127998,
            "range": "± 327",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 207298,
            "range": "± 300",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 207342,
            "range": "± 329",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 244958,
            "range": "± 545",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 255891,
            "range": "± 2521",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 335106,
            "range": "± 940",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 335117,
            "range": "± 785",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "252477aa6d45fbfbd2492bbaf5cbe1a73108bfe9",
          "message": "Merge pull request #864 from zcash/book-fixes-2\n\nMigrate to `mdbook 0.5`",
          "timestamp": "2025-12-04T12:15:57Z",
          "tree_id": "1f0ffad33e80c417cf8823577f8acf1b373d9f85",
          "url": "https://github.com/zcash/halo2/commit/252477aa6d45fbfbd2492bbaf5cbe1a73108bfe9"
        },
        "date": 1764851145100,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71939363,
            "range": "± 588399",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4063199,
            "range": "± 21168",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136837308,
            "range": "± 3352952",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4651898,
            "range": "± 98828",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 188083320,
            "range": "± 1037430",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5052242,
            "range": "± 64229",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31682,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132773,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145853,
            "range": "± 390",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 237396,
            "range": "± 1498",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 237341,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135508,
            "range": "± 1306",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148589,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 240089,
            "range": "± 778",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 239897,
            "range": "± 587",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 283777,
            "range": "± 2219",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 297170,
            "range": "± 581",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 388363,
            "range": "± 701",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 388339,
            "range": "± 753",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3e38720bdbdb0ed5cc18c2cbc187d81aecb45a42",
          "message": "Merge pull request #863 from zcash/dependabot/github_actions/zizmorcore/zizmor-action-0.3.0\n\nBump zizmorcore/zizmor-action from 0.2.0 to 0.3.0",
          "timestamp": "2025-12-04T15:40:00Z",
          "tree_id": "028f44f2097c655bf78a1b1728551e0dc1b77c0b",
          "url": "https://github.com/zcash/halo2/commit/3e38720bdbdb0ed5cc18c2cbc187d81aecb45a42"
        },
        "date": 1764863387078,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71846559,
            "range": "± 5720641",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4103041,
            "range": "± 86051",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136235786,
            "range": "± 3185314",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4675026,
            "range": "± 79168",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 187566820,
            "range": "± 994967",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5038027,
            "range": "± 39853",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31707,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132803,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145847,
            "range": "± 390",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 237768,
            "range": "± 604",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 237848,
            "range": "± 1090",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135494,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148491,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 240435,
            "range": "± 646",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 240339,
            "range": "± 592",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 283653,
            "range": "± 726",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 296617,
            "range": "± 462",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 388433,
            "range": "± 734",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 388532,
            "range": "± 4502",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "35ad82d29f124c29dc0d09ecfd52988ba47dcbe2",
          "message": "Merge pull request #865 from zcash/ci-fixes-3\n\nCI: Install required library before building rustdocs",
          "timestamp": "2025-12-04T16:50:38Z",
          "tree_id": "dc7d8d99d878cdda69f5ecc46cb64c245d4fbe26",
          "url": "https://github.com/zcash/halo2/commit/35ad82d29f124c29dc0d09ecfd52988ba47dcbe2"
        },
        "date": 1764867630400,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72127162,
            "range": "± 637574",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4080403,
            "range": "± 27124",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136739712,
            "range": "± 3061726",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4586010,
            "range": "± 108966",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 188313834,
            "range": "± 1328654",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5113618,
            "range": "± 54366",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31684,
            "range": "± 286",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 132984,
            "range": "± 3495",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145800,
            "range": "± 902",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 235853,
            "range": "± 3624",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 235955,
            "range": "± 638",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135835,
            "range": "± 767",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148709,
            "range": "± 2541",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 238813,
            "range": "± 864",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 238695,
            "range": "± 536",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 284500,
            "range": "± 1079",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 296893,
            "range": "± 718",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 387173,
            "range": "± 631",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 387215,
            "range": "± 1265",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "20da0ccadf62802cc8db7e45b261253ea3775c6b",
          "message": "Merge pull request #860 from zcash/dependabot/github_actions/peaceiris/actions-mdbook-2\n\nBump peaceiris/actions-mdbook from 1 to 2",
          "timestamp": "2025-12-04T17:04:21Z",
          "tree_id": "89b9b73c33b02cd69f17ea82e074e32a7925f6ce",
          "url": "https://github.com/zcash/halo2/commit/20da0ccadf62802cc8db7e45b261253ea3775c6b"
        },
        "date": 1764868439783,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71903387,
            "range": "± 966928",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4115644,
            "range": "± 29985",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136261714,
            "range": "± 3305417",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4643061,
            "range": "± 106764",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 187797316,
            "range": "± 1087563",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5021068,
            "range": "± 57923",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31667,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 133342,
            "range": "± 286",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 146398,
            "range": "± 717",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 235916,
            "range": "± 753",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 235798,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 136104,
            "range": "± 343",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 149164,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 238546,
            "range": "± 515",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 238520,
            "range": "± 1228",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 284752,
            "range": "± 1219",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 298177,
            "range": "± 659",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 386693,
            "range": "± 695",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 386516,
            "range": "± 2407",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}