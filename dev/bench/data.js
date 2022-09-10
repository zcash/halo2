window.BENCHMARK_DATA = {
  "lastUpdate": 1662811170528,
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
          "id": "553584a77f147061d4a2210ebd4ed25ec83af3ea",
          "message": "Merge pull request #644 from zcash/remove-rotated-poly-cache\n\nhalo2_proofs: Avoid caching rotated polynomials in `poly::Evaluator`",
          "timestamp": "2022-09-10T12:02:28+01:00",
          "tree_id": "da659bf51fedc5c65377594900fd9314de63ea34",
          "url": "https://github.com/zcash/halo2/commit/553584a77f147061d4a2210ebd4ed25ec83af3ea"
        },
        "date": 1662811163653,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 60393596,
            "range": "± 2803241",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 2945192,
            "range": "± 48224",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 131975483,
            "range": "± 2268638",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3651448,
            "range": "± 91792",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 183692813,
            "range": "± 2096365",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 3993870,
            "range": "± 133920",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38738,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141466,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153736,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 249924,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 250013,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144467,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156696,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 252903,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 252926,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 302176,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 314523,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 410828,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 410947,
            "range": "± 2013",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3108269,
            "range": "± 13963",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5886677,
            "range": "± 1852",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10186386,
            "range": "± 9339",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 20597492,
            "range": "± 53339",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 41359580,
            "range": "± 644819",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 82810357,
            "range": "± 330678",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7741,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9033,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15885,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19333,
            "range": "± 584",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27986,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 46739,
            "range": "± 916",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 97231,
            "range": "± 4474",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 187991,
            "range": "± 7936",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 384745,
            "range": "± 15709",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 813385,
            "range": "± 34542",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1720540,
            "range": "± 46280",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3684310,
            "range": "± 116373",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8092442,
            "range": "± 77225",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17626891,
            "range": "± 203942",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 38587794,
            "range": "± 751779",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 86141498,
            "range": "± 1078903",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28405,
            "range": "± 1002",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28529,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 152913028,
            "range": "± 395435",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 328236094,
            "range": "± 377401",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 707383981,
            "range": "± 1964540",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1518771648,
            "range": "± 4906615",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3257032573,
            "range": "± 8903864",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6966676613,
            "range": "± 11050348",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14826486489,
            "range": "± 19539552",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31539138934,
            "range": "± 44326001",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66640290178,
            "range": "± 72430400",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 95589355,
            "range": "± 1122696",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 164812701,
            "range": "± 785668",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 291859246,
            "range": "± 3250654",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 532260502,
            "range": "± 3673451",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 987799191,
            "range": "± 1878854",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1868514099,
            "range": "± 7413921",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3557286940,
            "range": "± 9750170",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6869988227,
            "range": "± 7510198",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13216595472,
            "range": "± 30048262",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5155708,
            "range": "± 40652",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8002268,
            "range": "± 49855",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12794042,
            "range": "± 518236",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21765346,
            "range": "± 628257",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37514650,
            "range": "± 300412",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 67085077,
            "range": "± 1771904",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120039105,
            "range": "± 532159",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222653410,
            "range": "± 6025238",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 404826191,
            "range": "± 2205823",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}