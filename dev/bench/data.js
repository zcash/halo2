window.BENCHMARK_DATA = {
  "lastUpdate": 1651808418599,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
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
          "id": "6e762bdde4b3d44dfe260c48dab592b8fd266463",
          "message": "Merge pull request #380 from 3for/main\n\n[book] Fix errors and typos",
          "timestamp": "2022-05-05T20:42:23-06:00",
          "tree_id": "5fec2be1e93eadf09bce1fc97c6046994fd7c034",
          "url": "https://github.com/zcash/halo2/commit/6e762bdde4b3d44dfe260c48dab592b8fd266463"
        },
        "date": 1651808414238,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 75743460,
            "range": "± 4965572",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3364864,
            "range": "± 86864",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 174227508,
            "range": "± 6589024",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4950982,
            "range": "± 115905",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 237178717,
            "range": "± 11266399",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5688518,
            "range": "± 150507",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 32641,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141237,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153409,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 245750,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 245630,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144187,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156418,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 248562,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 248637,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 301688,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 314005,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 406053,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 406137,
            "range": "± 642",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2977225,
            "range": "± 1986",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5883739,
            "range": "± 1841",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 8974059,
            "range": "± 17918",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 19475020,
            "range": "± 76102",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 36796969,
            "range": "± 349424",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 74409987,
            "range": "± 570745",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7056,
            "range": "± 537",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8158,
            "range": "± 1194",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14794,
            "range": "± 664",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 17335,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 24753,
            "range": "± 1110",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 45904,
            "range": "± 1696",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 92980,
            "range": "± 11799",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 166519,
            "range": "± 8707",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 383293,
            "range": "± 15329",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 814610,
            "range": "± 48972",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1664411,
            "range": "± 79866",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3569661,
            "range": "± 88181",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7859717,
            "range": "± 185464",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17420178,
            "range": "± 1447687",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 38063939,
            "range": "± 1029559",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 86977761,
            "range": "± 3254752",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28445,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28526,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 150976930,
            "range": "± 3354582",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 327362110,
            "range": "± 1263830",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 698242753,
            "range": "± 5742422",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1508743882,
            "range": "± 21281220",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3228362237,
            "range": "± 90819459",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6760023034,
            "range": "± 319024373",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14400223247,
            "range": "± 748414621",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31408018856,
            "range": "± 298412840",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 65677272562,
            "range": "± 5731752151",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 96073834,
            "range": "± 2318394",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 165388652,
            "range": "± 818923",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 293923928,
            "range": "± 1883804",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 529230550,
            "range": "± 6803956",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1002725377,
            "range": "± 3547093",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1899544804,
            "range": "± 14645481",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3597341978,
            "range": "± 14931790",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6974080309,
            "range": "± 38041036",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13459145586,
            "range": "± 48243426",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5173668,
            "range": "± 63509",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7965982,
            "range": "± 107826",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 13046883,
            "range": "± 461536",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21999117,
            "range": "± 191579",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37833186,
            "range": "± 439676",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 67612874,
            "range": "± 755388",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 106724049,
            "range": "± 480916",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 221208236,
            "range": "± 4141182",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 401626334,
            "range": "± 2264404",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}