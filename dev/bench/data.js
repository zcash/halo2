window.BENCHMARK_DATA = {
  "lastUpdate": 1665228156436,
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
          "id": "0f7299c1165e98d1e26c73fabd6ab9429b64559e",
          "message": "Merge pull request #625 from zcash/region-query-instance\n\nhalo2_proofs: Introduce `RegionLayouter::instance_value` method.",
          "timestamp": "2022-10-08T11:18:46+01:00",
          "tree_id": "ffca7af06a8932ce4d740e7dd1d4ab39191d6433",
          "url": "https://github.com/zcash/halo2/commit/0f7299c1165e98d1e26c73fabd6ab9429b64559e"
        },
        "date": 1665228149810,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 66851455,
            "range": "± 5273851",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3455273,
            "range": "± 159856",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 141345039,
            "range": "± 3824245",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4200603,
            "range": "± 86623",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 199229396,
            "range": "± 1348321",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4641767,
            "range": "± 97497",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 47886,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 164197,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 179384,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 294480,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 294444,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 167639,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 182826,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 297825,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 297742,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 350801,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 365957,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 481037,
            "range": "± 416",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 481320,
            "range": "± 1380",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3490297,
            "range": "± 2628",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6065578,
            "range": "± 5044",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10435799,
            "range": "± 47318",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22807227,
            "range": "± 93121",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 42893270,
            "range": "± 84799",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 81484933,
            "range": "± 163450",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7222,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9383,
            "range": "± 819",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15472,
            "range": "± 489",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 20661,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 29336,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 49379,
            "range": "± 1474",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 102610,
            "range": "± 11605",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 199079,
            "range": "± 13112",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 405705,
            "range": "± 8447",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 850787,
            "range": "± 15564",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1791245,
            "range": "± 19209",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3791903,
            "range": "± 59088",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8093230,
            "range": "± 178791",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17460013,
            "range": "± 173640",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 37328118,
            "range": "± 323516",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 83851515,
            "range": "± 1834068",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34790,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34921,
            "range": "± 663",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 176869513,
            "range": "± 11078434",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 380813676,
            "range": "± 5419340",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 821364769,
            "range": "± 1741821",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1769159936,
            "range": "± 7876361",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3789216070,
            "range": "± 12359219",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8088032208,
            "range": "± 9463647",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17251322551,
            "range": "± 24536559",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36597606448,
            "range": "± 14843837",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 77392349534,
            "range": "± 37462202",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 109560367,
            "range": "± 689728",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 189074552,
            "range": "± 1565105",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 336674530,
            "range": "± 2669747",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 608418262,
            "range": "± 1866554",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1124799123,
            "range": "± 2927586",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2134871869,
            "range": "± 61280969",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4058409271,
            "range": "± 11757718",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7821020935,
            "range": "± 16052633",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15078783707,
            "range": "± 41702199",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5862172,
            "range": "± 88234",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9120501,
            "range": "± 197195",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14885881,
            "range": "± 775073",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24862069,
            "range": "± 1135111",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 42726037,
            "range": "± 333675",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 76454241,
            "range": "± 2962343",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 137258212,
            "range": "± 600079",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 252683207,
            "range": "± 9369649",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 458904051,
            "range": "± 3382060",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}