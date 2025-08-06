window.BENCHMARK_DATA = {
  "lastUpdate": 1754480926714,
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
      }
    ]
  }
}