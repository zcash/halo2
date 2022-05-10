window.BENCHMARK_DATA = {
  "lastUpdate": 1652223170826,
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
          "id": "50921f95f7d4edf48f79ffba3b892a983d91ed7f",
          "message": "Merge pull request #573 from zcash/str4d/chip-review\n\nChanges from `halo2_gadgets` review",
          "timestamp": "2022-05-10T22:54:45+01:00",
          "tree_id": "58f6bfa11eb3602650e308cf35988358a64b9ea4",
          "url": "https://github.com/zcash/halo2/commit/50921f95f7d4edf48f79ffba3b892a983d91ed7f"
        },
        "date": 1652223166445,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 74075734,
            "range": "± 3367510",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3268829,
            "range": "± 56486",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 173352773,
            "range": "± 6335467",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4791217,
            "range": "± 417237",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 220755346,
            "range": "± 3116930",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5924371,
            "range": "± 97665",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 32371,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 126422,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 136677,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 221788,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 221683,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 129204,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 139358,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 224404,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 224452,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 270281,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 279877,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 364485,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 364524,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2756672,
            "range": "± 1091",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5192395,
            "range": "± 3185",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 9012742,
            "range": "± 9279",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 20384021,
            "range": "± 605247",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 38490011,
            "range": "± 156527",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 75735267,
            "range": "± 284623",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7270,
            "range": "± 428",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8799,
            "range": "± 1236",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15311,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 16864,
            "range": "± 740",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 25751,
            "range": "± 961",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 44966,
            "range": "± 1093",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 93162,
            "range": "± 5521",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 185885,
            "range": "± 4531",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 383549,
            "range": "± 19207",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 798442,
            "range": "± 18363",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1669484,
            "range": "± 29741",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3581125,
            "range": "± 29520",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7947104,
            "range": "± 599778",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17379654,
            "range": "± 212521",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 38402111,
            "range": "± 117621",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 87026970,
            "range": "± 753102",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 25132,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 25191,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 151282765,
            "range": "± 5800738",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 321619506,
            "range": "± 797111",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 692923498,
            "range": "± 4808126",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1492368216,
            "range": "± 11349742",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3199630355,
            "range": "± 24682983",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6836048946,
            "range": "± 37948241",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14571462527,
            "range": "± 86321149",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 30978968714,
            "range": "± 38249807",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 65623542564,
            "range": "± 917556697",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 94238015,
            "range": "± 638082",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 162874006,
            "range": "± 1091145",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 290144656,
            "range": "± 961301",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 531554862,
            "range": "± 2309781",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 996304333,
            "range": "± 6497555",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1883750863,
            "range": "± 4020574",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3574379008,
            "range": "± 4546050",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6920147039,
            "range": "± 11460708",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13323950514,
            "range": "± 27603663",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5044334,
            "range": "± 22078",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7803652,
            "range": "± 20505",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12805226,
            "range": "± 91711",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21809472,
            "range": "± 70814",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37319620,
            "range": "± 1355350",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66319934,
            "range": "± 438728",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 118661359,
            "range": "± 2392633",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 221119060,
            "range": "± 4542089",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 401234204,
            "range": "± 3999592",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}