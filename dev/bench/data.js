window.BENCHMARK_DATA = {
  "lastUpdate": 1678502852334,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "642924d614305d882cc122739c59144109f4bd3f",
          "message": "Merge pull request #739 from daira/fix-sort-nondeterminism\n\nFix a nondeterminism bug: we were depending on sort order",
          "timestamp": "2023-03-11T01:43:52Z",
          "tree_id": "5a6d57e447268d912d70b9d6cb9e23d60357a833",
          "url": "https://github.com/zcash/halo2/commit/642924d614305d882cc122739c59144109f4bd3f"
        },
        "date": 1678502844201,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 98524887,
            "range": "± 8481064",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 5634030,
            "range": "± 603127",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 183572572,
            "range": "± 4737769",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6372115,
            "range": "± 624611",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 249431408,
            "range": "± 6233952",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7567141,
            "range": "± 687035",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 46407,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 160497,
            "range": "± 715",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 175744,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 284230,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 284114,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 163857,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 179052,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 287400,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 287389,
            "range": "± 1121",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 342676,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 357991,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 466336,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 466290,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3416003,
            "range": "± 1291",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5901482,
            "range": "± 13679",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10129651,
            "range": "± 17423",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22485840,
            "range": "± 146490",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 39108054,
            "range": "± 201163",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 75956865,
            "range": "± 214431",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 6922,
            "range": "± 540",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8056,
            "range": "± 776",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14179,
            "range": "± 634",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18998,
            "range": "± 596",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26119,
            "range": "± 3207",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 44630,
            "range": "± 6972",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 104442,
            "range": "± 14346",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 216895,
            "range": "± 35753",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 470876,
            "range": "± 73451",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 897476,
            "range": "± 103423",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1740113,
            "range": "± 111997",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3424069,
            "range": "± 384585",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8181118,
            "range": "± 620755",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15680883,
            "range": "± 1894318",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 33835454,
            "range": "± 3878486",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 74715046,
            "range": "± 1220922",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34519,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34621,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 174225470,
            "range": "± 6898094",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 374377638,
            "range": "± 2794133",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 814577763,
            "range": "± 6626346",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1729915009,
            "range": "± 4782624",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3705401719,
            "range": "± 9070207",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 7926959612,
            "range": "± 11670163",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 16857975828,
            "range": "± 17785677",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 35787135919,
            "range": "± 30293829",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 75647757249,
            "range": "± 76737464",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 115759905,
            "range": "± 1797342",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 204813029,
            "range": "± 5630505",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 351236432,
            "range": "± 7731767",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 625022248,
            "range": "± 10392462",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1124796499,
            "range": "± 19399394",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2115680811,
            "range": "± 10256032",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4031748851,
            "range": "± 23215214",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7744141540,
            "range": "± 24172128",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 14841259389,
            "range": "± 50380378",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5803798,
            "range": "± 405670",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8983341,
            "range": "± 651378",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14396631,
            "range": "± 848837",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24311081,
            "range": "± 2594590",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 41871919,
            "range": "± 2634794",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 74951356,
            "range": "± 4440697",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 134767552,
            "range": "± 6098992",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 247948922,
            "range": "± 17639902",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 453498612,
            "range": "± 6099239",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}