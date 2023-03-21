window.BENCHMARK_DATA = {
  "lastUpdate": 1679373622793,
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
          "id": "1a53c0cbfb714ec65bdd150d8dd499d6f68660e8",
          "message": "Merge pull request #752 from zcash/cost-without-permutation\n\n`dev::cost`: Handle cost calculation for circuits without permutation",
          "timestamp": "2023-03-21T03:28:59Z",
          "tree_id": "33e8c8f187bc60f5d1bdbec923157b9ed4a0e8cf",
          "url": "https://github.com/zcash/halo2/commit/1a53c0cbfb714ec65bdd150d8dd499d6f68660e8"
        },
        "date": 1679373614146,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 118653177,
            "range": "± 14100933",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 6600338,
            "range": "± 693753",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 216670528,
            "range": "± 10940304",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 8105675,
            "range": "± 876817",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 295935971,
            "range": "± 11832753",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 8859129,
            "range": "± 1032876",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 52362,
            "range": "± 3911",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 175214,
            "range": "± 6045",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 190375,
            "range": "± 7879",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 310938,
            "range": "± 16088",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 309304,
            "range": "± 13711",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 179707,
            "range": "± 7452",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 186220,
            "range": "± 8167",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 298050,
            "range": "± 13866",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 309450,
            "range": "± 15984",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 375852,
            "range": "± 24467",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 398142,
            "range": "± 28014",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 524913,
            "range": "± 41358",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 500701,
            "range": "± 29621",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3761087,
            "range": "± 163488",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7237688,
            "range": "± 207272",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12858004,
            "range": "± 295056",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 27975872,
            "range": "± 1491159",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 51943657,
            "range": "± 1095653",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 102265165,
            "range": "± 2214202",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8326,
            "range": "± 704",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10179,
            "range": "± 683",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 18092,
            "range": "± 1955",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 23495,
            "range": "± 3337",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 32464,
            "range": "± 4185",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 57689,
            "range": "± 11797",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 134555,
            "range": "± 30992",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 277701,
            "range": "± 55895",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 532249,
            "range": "± 84803",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1030258,
            "range": "± 164964",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2125110,
            "range": "± 210672",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4581099,
            "range": "± 524310",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 9608828,
            "range": "± 1191914",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 21220389,
            "range": "± 2340344",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 47114195,
            "range": "± 2870303",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 97045579,
            "range": "± 4920055",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 32679,
            "range": "± 4169",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 33138,
            "range": "± 1772",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 196436718,
            "range": "± 4221090",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 442606343,
            "range": "± 4995500",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 926788255,
            "range": "± 20530056",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 2006950905,
            "range": "± 29539194",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4100762154,
            "range": "± 66189635",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8807386054,
            "range": "± 148741975",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 18630835146,
            "range": "± 318794772",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 40590810117,
            "range": "± 896384632",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 83088158172,
            "range": "± 1551424755",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 130466734,
            "range": "± 4963760",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 237278102,
            "range": "± 4340922",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 407019468,
            "range": "± 11546923",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 709624667,
            "range": "± 16244846",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1302518899,
            "range": "± 23055966",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2533544709,
            "range": "± 102804018",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4742776972,
            "range": "± 37444957",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8921169222,
            "range": "± 116231827",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 17250063599,
            "range": "± 281264309",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6513967,
            "range": "± 702090",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9713383,
            "range": "± 1139474",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 16617940,
            "range": "± 2163928",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 30080613,
            "range": "± 3950183",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 48555408,
            "range": "± 4529509",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 81218848,
            "range": "± 5904572",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 153051187,
            "range": "± 8300755",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 290144214,
            "range": "± 21285106",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 525188851,
            "range": "± 12978885",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}