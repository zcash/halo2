window.BENCHMARK_DATA = {
  "lastUpdate": 1651766818546,
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
          "id": "b2e2b9b08125d806cb5836bd7c7dea0955febca5",
          "message": "Merge pull request #568 from zcash/circuit-review\n\nChanges from Orchard circuit review",
          "timestamp": "2022-05-05T16:03:31+01:00",
          "tree_id": "f78c68a981c6348eaa76a08589aa718de3d09f22",
          "url": "https://github.com/zcash/halo2/commit/b2e2b9b08125d806cb5836bd7c7dea0955febca5"
        },
        "date": 1651766815157,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 77460989,
            "range": "± 6496956",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3377097,
            "range": "± 62473",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 176434382,
            "range": "± 6917323",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4917415,
            "range": "± 57139",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 242688040,
            "range": "± 1438665",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6005402,
            "range": "± 161293",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 36584,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141707,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153950,
            "range": "± 812",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 251688,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 251686,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144677,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156883,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 254556,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 254607,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 302667,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 315013,
            "range": "± 385",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 412822,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 412813,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3267599,
            "range": "± 1549",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5938277,
            "range": "± 5866",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10939923,
            "range": "± 205603",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22089666,
            "range": "± 110062",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 41356081,
            "range": "± 519595",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 83244427,
            "range": "± 285000",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 14542,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 16765,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 20626,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 28368,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 44994,
            "range": "± 1167",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 80404,
            "range": "± 416",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 155300,
            "range": "± 1228",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 312634,
            "range": "± 17527",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 655242,
            "range": "± 12907",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1408417,
            "range": "± 52437",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2977695,
            "range": "± 98197",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 6264310,
            "range": "± 79445",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 13216829,
            "range": "± 444638",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 28416923,
            "range": "± 315446",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 60643221,
            "range": "± 1081873",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 125571391,
            "range": "± 818288",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28442,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28541,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 192764490,
            "range": "± 435554",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 407426712,
            "range": "± 1581840",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 867297101,
            "range": "± 940235",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1838654865,
            "range": "± 6041659",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3901530930,
            "range": "± 9221286",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8234894769,
            "range": "± 6515957",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17386895421,
            "range": "± 25348047",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36609735722,
            "range": "± 93456335",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 76928299842,
            "range": "± 57078416",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 97040301,
            "range": "± 556064",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 167572355,
            "range": "± 821832",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 299603497,
            "range": "± 4384221",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 545172045,
            "range": "± 2585399",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1022249361,
            "range": "± 2933325",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1929212828,
            "range": "± 12015638",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3671174135,
            "range": "± 9815353",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7086880651,
            "range": "± 26413090",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13678229701,
            "range": "± 34940718",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5197126,
            "range": "± 31481",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8005070,
            "range": "± 119710",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12803202,
            "range": "± 126977",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 22011204,
            "range": "± 133203",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37880042,
            "range": "± 270639",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66365966,
            "range": "± 498946",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120335946,
            "range": "± 305457",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222340225,
            "range": "± 2586819",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 402977440,
            "range": "± 2148433",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}