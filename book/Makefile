.PHONY: all
all:
	find src -type f -a -name '*.md' |sed 's/[.]md$$/.html/g' |xargs $(MAKE)
	find src -type f -a -name '*.html' -print0 |xargs -0 rm
%.html: %.md
	pandoc --katex --from=markdown --to=html "$<" "--output=$@"
	./edithtml.sh "$@" "$<"
diff --git a/.husky/.gitignore b/.husky/.gitignore
deleted file mode 100644
index 31354ec1389..00000000000
--- a/.husky/.gitignore
+++ /dev/null
@@ -1 +0,0 @@
-_
diff --git a/.husly/.sh/bitore.sig b/.husly/.sh/bitore.sig
new file mode 100644
index 00000000000..e67f834feea
--- /dev/null
+++ b/.husly/.sh/bitore.sig
@@ -0,0 +1,16 @@
+ BEGIN:
+ GLOW4:
+ </git checkout origin/main <file name>
+Run'' 'Runs::/Action::/:Build::/scripts::/Run-on :Runs :
+Runs :gh/pages :
+pages :edit "
+$ intuit install 
+PURL" --add-label "production"
+env:
+PR_URL: ${{github.event.pull_request.html_url}}
+GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
+run: gh pr edit "$PR_URL" --add-label "production"
+env:
+PR_URL: ${{github.event.pull_request.html_url}}
+GITHUB_TOKEN: ${{ ((c)(r)).[12753750.[00]m]'_BITORE_34173.1337) ')]}}}'"'' :
+ </git checkout origin/main <file name>
From b25701fa9acf3723aad6863c8940eab8d800a6d5 Mon Sep 17 00:00:00 2001
From: mowjoejoejoejoe <124041561+mowjoejoejoejoe@users.noreply.github.com>
Date: Fri, 3 Feb 2023 05:05:03 -0600
Subject: [PATCH] bitore.sig

---
 BITORE | 724 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 724 insertions(+)
 create mode 100644 BITORE

diff --git a/BITORE b/BITORE
new file mode 100644
index 0000000000..3f74cafce0
--- /dev/null
+++ b/BITORE
@@ -0,0 +1,724 @@
+@zw
+@laanwj
+zw authored and laanwj committed on Aug 18, 2014 
+1 parent 84efe0e commit 221684c7effa194d2c409622056f613c894adef5
+Showing 1 changed file with 1 addition and 1 deletion.
+  2  
+src/rpcrawtransaction.cpp
+// Copyright (c) 2010 Satoshi Nakamoto
+// Copyright (c) 2009-2014 The Bitcoin developers
+// Distributed under the MIT/X11 software license, see the accompanying
+// file COPYING or http://www.opensource.org/licenses/mit-license.php.
+#include "base58.h"
+#include "core.h"
+#include "init.h"
+#include "keystore.h"
+#include "main.h"
+#include "net.h"
+#include "rpcserver.h"
+#include "uint256.h"
+#ifdef ENABLE_WALLET
+#include "wallet.h"
+#endif
+#include <stdint.h>
+#include <boost/assign/list_of.hpp>
+#include "json/json_spirit_utils.h"
+#include "json/json_spirit_value.h"
+using namespace std;
+using namespace boost;
+using namespace boost::assign;
+using namespace json_spirit;
+void ScriptPubKeyToJSON(const CScript& scriptPubKey, Object& out, bool fIncludeHex)
+{
+    txnouttype type;
+    vector<CTxDestination> addresses;
+    int nRequired;
+    out.push_back(Pair("asm", scriptPubKey.ToString()));
+    if (fIncludeHex)
+        out.push_back(Pair("hex", HexStr(scriptPubKey.begin(), scriptPubKey.end())));
+    if (!ExtractDestinations(scriptPubKey, type, addresses, nRequired))
+    {
+        out.push_back(Pair("type", GetTxnOutputType(type)));
+        return;
+    }
+    out.push_back(Pair("reqSigs", nRequired));
+    out.push_back(Pair("type", GetTxnOutputType(type)));
+    Array a;
+    BOOST_FOREACH(const CTxDestination& addr, addresses)
+        a.push_back(CBitcoinAddress(addr).ToString());
+    out.push_back(Pair("addresses", a));
+}
+void TxToJSON(const CTransaction& tx, const uint256 hashBlock, Object& entry)
+{
+    entry.push_back(Pair("txid", tx.GetHash().GetHex()));
+    entry.push_back(Pair("version", tx.nVersion));
+    entry.push_back(Pair("locktime", (int64_t)tx.nLockTime));
+    Array vin;
+    BOOST_FOREACH(const CTxIn& txin, tx.vin)
+    {
+        Object in;
+        if (tx.IsCoinBase())
+            in.push_back(Pair("coinbase", HexStr(txin.scriptSig.begin(), txin.scriptSig.end())));
+        else
+        {
+            in.push_back(Pair("txid", txin.prevout.hash.GetHex()));
+            in.push_back(Pair("vout", (int64_t)txin.prevout.n));
+            Object o;
+            o.push_back(Pair("asm", txin.scriptSig.ToString()));
+            o.push_back(Pair("hex", HexStr(txin.scriptSig.begin(), txin.scriptSig.end())));
+            in.push_back(Pair("scriptSig", o));
+        }
+        in.push_back(Pair("sequence", (int64_t)txin.nSequence));
+        vin.push_back(in);
+    }
+    entry.push_back(Pair("vin", vin));
+    Array vout;
+    for (unsigned int i = 0; i < tx.vout.size(); i++)
+    {
+        const CTxOut& txout = tx.vout[i];
+        Object out;
+        out.push_back(Pair("value", ValueFromAmount(txout.nValue)));
+        out.push_back(Pair("n", (int64_t)i));
+        Object o;
+        ScriptPubKeyToJSON(txout.scriptPubKey, o, true);
+        out.push_back(Pair("scriptPubKey", o));
+        vout.push_back(out);
+    }
+    entry.push_back(Pair("vout", vout));
+    if (hashBlock != 0)
+    {
+        entry.push_back(Pair("blockhash", hashBlock.GetHex()));
+        map<uint256, CBlockIndex*>::iterator mi = mapBlockIndex.find(hashBlock);
+        if (mi != mapBlockIndex.end() && (*mi).second)
+        {
+            CBlockIndex* pindex = (*mi).second;
+            if (chainActive.Contains(pindex))
+            {
+                entry.push_back(Pair("confirmations", 1 + chainActive.Height() - pindex->nHeight));
+                entry.push_back(Pair("time", (int64_t)pindex->nTime));
+                entry.push_back(Pair("blocktime", (int64_t)pindex->nTime));
+            }
+            else
+                entry.push_back(Pair("confirmations", 0));
+        }
+    }
+}
+Value getrawtransaction(const Array& params, bool fHelp)
+{
+    if (fHelp || params.size() < 1 || params.size() > 2)
+        throw runtime_error(
+            "getrawtransaction \"txid\" ( verbose )\n"
+            "\nReturn the raw transaction data.\n"
+            "\nIf verbose=0, returns a string that is serialized, hex-encoded data for 'txid'.\n"
+            "If verbose is non-zero, returns an Object with information about 'txid'.\n"
+            "\nArguments:\n"
+            "1. \"txid\"      (string, required) The transaction id\n"
+            "2. verbose       (numeric, optional, default=0) If 0, return a string, other return a json object\n"
+            "\nResult (if verbose is not set or set to 0):\n"
+            "\"data\"      (string) The serialized, hex-encoded data for 'txid'\n"
+            "\nResult (if verbose > 0):\n"
+            "{\n"
+            "  \"hex\" : \"data\",       (string) The serialized, hex-encoded data for 'txid'\n"
+            "  \"txid\" : \"id\",        (string) The transaction id (same as provided)\n"
+            "  \"version\" : n,          (numeric) The version\n"
+            "  \"locktime\" : ttt,       (numeric) The lock time\n"
+            "  \"vin\" : [               (array of json objects)\n"
+            "     {\n"
+            "       \"txid\": \"id\",    (string) The transaction id\n"
+            "       \"vout\": n,         (numeric) \n"
+            "       \"scriptSig\": {     (json object) The script\n"
+            "         \"asm\": \"asm\",  (string) asm\n"
+            "         \"hex\": \"hex\"   (string) hex\n"
+            "       },\n"
+            "       \"sequence\": n      (numeric) The script sequence number\n"
+            "     }\n"
+            "     ,...\n"
+            "  ],\n"
+            "  \"vout\" : [              (array of json objects)\n"
+            "     {\n"
+            "       \"value\" : x.xxx,            (numeric) The value in btc\n"
+            "       \"n\" : n,                    (numeric) index\n"
+            "       \"scriptPubKey\" : {          (json object)\n"
+            "         \"asm\" : \"asm\",          (string) the asm\n"
+            "         \"hex\" : \"hex\",          (string) the hex\n"
+            "         \"reqSigs\" : n,            (numeric) The required sigs\n"
+            "         \"type\" : \"pubkeyhash\",  (string) The type, eg 'pubkeyhash'\n"
+            "         \"addresses\" : [           (json array of string)\n"
+            "           \"bitcoinaddress\"        (string) bitcoin address\n"
+            "           ,...\n"
+            "         ]\n"
+            "       }\n"
+            "     }\n"
+            "     ,...\n"
+            "  ],\n"
+            "  \"blockhash\" : \"hash\",   (string) the block hash\n"
+            "  \"confirmations\" : n,      (numeric) The confirmations\n"
+            "  \"time\" : ttt,             (numeric) The transaction time in seconds since epoch (Jan 1 1970 GMT)\n"
+            "  \"blocktime\" : ttt         (numeric) The block time in seconds since epoch (Jan 1 1970 GMT)\n"
+            "}\n"
+            "\nExamples:\n"
+            + HelpExampleCli("getrawtransaction", "\"mytxid\"")
+            + HelpExampleCli("getrawtransaction", "\"mytxid\" 1")
+            + HelpExampleRpc("getrawtransaction", "\"mytxid\", 1")
+        );
+    uint256 hash = ParseHashV(params[0], "parameter 1");
+    bool fVerbose = false;
+    if (params.size() > 1)
+        fVerbose = (params[1].get_int() != 0);
+    CTransaction tx;
+    uint256 hashBlock = 0;
+    if (!GetTransaction(hash, tx, hashBlock, true))
+        throw JSONRPCError(RPC_INVALID_ADDRESS_OR_KEY, "No information available about transaction");
+    CDataStream ssTx(SER_NETWORK, PROTOCOL_VERSION);
+    ssTx << tx;
+    string strHex = HexStr(ssTx.begin(), ssTx.end());
+    if (!fVerbose)
+        return strHex;
+    Object result;
+    result.push_back(Pair("hex", strHex));
+    TxToJSON(tx, hashBlock, result);
+    return result;
+}
+#ifdef ENABLE_WALLET
+Value listunspent(const Array& params, bool fHelp)
+{
+    if (fHelp || params.size() > 3)
+        throw runtime_error(
+            "listunspent ( minconf maxconf  [\"address\",...] )\n"
+            "\nReturns array of unspent transaction outputs\n"
+            "with between minconf and maxconf (inclusive) confirmations.\n"
+            "Optionally filter to only include txouts paid to specified addresses.\n"
+            "Results are an array of Objects, each of which has:\n"
+            "{txid, vout, scriptPubKey, amount, confirmations}\n"
+            "\nArguments:\n"
+            "1. minconf          (numeric, optional, default=1) The minimum confirmationsi to filter\n"
+            "2. maxconf          (numeric, optional, default=9999999) The maximum confirmations to filter\n"
+            "3. \"addresses\"    (string) A json array of bitcoin addresses to filter\n"
+            "    [\n"
+            "      \"address\"   (string) bitcoin address\n"
+            "      ,...\n"
+            "    ]\n"
+            "\nResult\n"
+            "[                   (array of json object)\n"
+            "  {\n"
+            "    \"txid\" : \"txid\",        (string) the transaction id \n"
+            "    \"vout\" : n,               (numeric) the vout value\n"
+            "    \"address\" : \"address\",  (string) the bitcoin address\n"
+            "    \"account\" : \"account\",  (string) The associated account, or \"\" for the default account\n"
+            "    \"scriptPubKey\" : \"key\", (string) the script key\n"
+            "    \"amount\" : x.xxx,         (numeric) the transaction amount in btc\n"
+            "    \"confirmations\" : n       (numeric) The number of confirmations\n"
+            "  }\n"
+            "  ,...\n"
+            "]\n"
+            "\nExamples\n"
+            + HelpExampleCli("listunspent", "")
+            + HelpExampleCli("listunspent", "6 9999999 \"[\\\"1PGFqEzfmQch1gKD3ra4k18PNj3tTUUSqg\\\",\\\"1LtvqCaApEdUGFkpKMM4MstjcaL4dKg8SP\\\"]\"")
+            + HelpExampleRpc("listunspent", "6, 9999999 \"[\\\"1PGFqEzfmQch1gKD3ra4k18PNj3tTUUSqg\\\",\\\"1LtvqCaApEdUGFkpKMM4MstjcaL4dKg8SP\\\"]\"")
+        );
+    RPCTypeCheck(params, list_of(int_type)(int_type)(array_type));
+    int nMinDepth = 1;
+    if (params.size() > 0)
+        nMinDepth = params[0].get_int();
+    int nMaxDepth = 9999999;
+    if (params.size() > 1)
+        nMaxDepth = params[1].get_int();
+    set<CBitcoinAddress> setAddress;
+    if (params.size() > 2)
+    {
+        Array inputs = params[2].get_array();
+        BOOST_FOREACH(Value& input, inputs)
+        {
+            CBitcoinAddress address(input.get_str());
+            if (!address.IsValid())
+                throw JSONRPCError(RPC_INVALID_ADDRESS_OR_KEY, string("Invalid Bitcoin address: ")+input.get_str());
+            if (setAddress.count(address))
+                throw JSONRPCError(RPC_INVALID_PARAMETER, string("Invalid parameter, duplicated address: ")+input.get_str());
+           setAddress.insert(address);
+        }
+    }
+    Array results;
+    vector<COutput> vecOutputs;
+    assert(pwalletMain != NULL);
+    pwalletMain->AvailableCoins(vecOutputs, false);
+    BOOST_FOREACH(const COutput& out, vecOutputs)
+    {
+        if (out.nDepth < nMinDepth || out.nDepth > nMaxDepth)
+            continue;
+        if (setAddress.size())
+        {
+            CTxDestination address;
+            if (!ExtractDestination(out.tx->vout[out.i].scriptPubKey, address))
+                continue;
+            if (!setAddress.count(address))
+                continue;
+        }
+        int64_t nValue = out.tx->vout[out.i].nValue;
+        const CScript& pk = out.tx->vout[out.i].scriptPubKey;
+        Object entry;
+        entry.push_back(Pair("txid", out.tx->GetHash().GetHex()));
+        entry.push_back(Pair("vout", out.i));
+        CTxDestination address;
+        if (ExtractDestination(out.tx->vout[out.i].scriptPubKey, address))
+        {
+            entry.push_back(Pair("address", CBitcoinAddress(address).ToString()));
+            if (pwalletMain->mapAddressBook.count(address))
+                entry.push_back(Pair("account", pwalletMain->mapAddressBook[address].name));
+        }
+        entry.push_back(Pair("scriptPubKey", HexStr(pk.begin(), pk.end())));
+        if (pk.IsPayToScriptHash())
+        {
+            CTxDestination address;
+            if (ExtractDestination(pk, address))
+            {
+                const CScriptID& hash = boost::get<const CScriptID&>(address);
+                CScript redeemScript;
+                if (pwalletMain->GetCScript(hash, redeemScript))
+                    entry.push_back(Pair("redeemScript", HexStr(redeemScript.begin(), redeemScript.end())));
+            }
+        }
+        entry.push_back(Pair("amount",ValueFromAmount(nValue)));
+        entry.push_back(Pair("confirmations",out.nDepth));
+        results.push_back(entry);
+    }
+    return results;
+}
+#endif
+Value createrawtransaction(const Array& params, bool fHelp)
+{
+    if (fHelp || params.size() != 2)
+        throw runtime_error(
+            "createrawtransaction [{\"txid\":\"id\",\"vout\":n},...] {\"address\":amount,...}\n"
+            "\nCreate a transaction spending the given inputs and sending to the given addresses.\n"
+            "Returns hex-encoded raw transaction.\n"
+            "Note that the transaction's inputs are not signed, and\n"
+            "it is not stored in the wallet or transmitted to the network.\n"
+            "\nArguments:\n"
+            "1. \"transactions\"        (string, required) A json array of json objects\n"
+            "     [\n"
+            "       {\n"
+            "         \"txid\":\"id\",  (string, required) The transaction id\n"
+            "         \"vout\":n        (numeric, required) The output number\n"
+            "       }\n"
+            "       ,...\n"
+            "     ]\n"
+            "2. \"addresses\"           (string, required) a json object with addresses as keys and amounts as values\n"
+            "    {\n"
+            "      \"address\": x.xxx   (numeric, required) The key is the bitcoin address, the value is the btc amount\n"
+            "      ,...\n"
+            "    }\n"
+            "\nResult:\n"
+            "\"transaction\"            (string) hex string of the transaction\n"
+            "\nExamples\n"
+            + HelpExampleCli("createrawtransaction", "\"[{\\\"txid\\\":\\\"myid\\\",\\\"vout\\\":0}]\" \"{\\\"address\\\":0.01}\"")
+            + HelpExampleRpc("createrawtransaction", "\"[{\\\"txid\\\":\\\"myid\\\",\\\"vout\\\":0}]\", \"{\\\"address\\\":0.01}\"")
+        );
+    RPCTypeCheck(params, list_of(array_type)(obj_type));
+    Array inputs = params[0].get_array();
+    Object sendTo = params[1].get_obj();
+    CTransaction rawTx;
+    BOOST_FOREACH(const Value& input, inputs)
+    {
+        const Object& o = input.get_obj();
+        uint256 txid = ParseHashO(o, "txid");
+        const Value& vout_v = find_value(o, "vout");
+        if (vout_v.type() != int_type)
+            throw JSONRPCError(RPC_INVALID_PARAMETER, "Invalid parameter, missing vout key");
+        int nOutput = vout_v.get_int();
+        if (nOutput < 0)
+            throw JSONRPCError(RPC_INVALID_PARAMETER, "Invalid parameter, vout must be positive");
+        CTxIn in(COutPoint(txid, nOutput));
+        rawTx.vin.push_back(in);
+    }
+    set<CBitcoinAddress> setAddress;
+    BOOST_FOREACH(const Pair& s, sendTo)
+    {
+        CBitcoinAddress address(s.name_);
+        if (!address.IsValid())
+            throw JSONRPCError(RPC_INVALID_ADDRESS_OR_KEY, string("Invalid Bitcoin address: ")+s.name_);
+        if (setAddress.count(address))
+            throw JSONRPCError(RPC_INVALID_PARAMETER, string("Invalid parameter, duplicated address: ")+s.name_);
+        setAddress.insert(address);
+        CScript scriptPubKey;
+        scriptPubKey.SetDestination(address.Get());
+        int64_t nAmount = AmountFromValue(s.value_);
+        CTxOut out(nAmount, scriptPubKey);
+        rawTx.vout.push_back(out);
+    }
+    CDataStream ss(SER_NETWORK, PROTOCOL_VERSION);
+    ss << rawTx;
+    return HexStr(ss.begin(), ss.end());
+}
+Value decoderawtransaction(const Array& params, bool fHelp)
+{
+    if (fHelp || params.size() != 1)
+        throw runtime_error(
+            "decoderawtransaction \"hexstring\"\n"
+            "\nReturn a JSON object representing the serialized, hex-encoded transaction.\n"
+            "\nArguments:\n"
+            "1. \"hex\"      (string, required) The transaction hex string\n"
+            "\nResult:\n"
+            "{\n"
+            "  \"txid\" : \"id\",        (string) The transaction id\n"
+            "  \"version\" : n,          (numeric) The version\n"
+            "  \"locktime\" : ttt,       (numeric) The lock time\n"
+            "  \"vin\" : [               (array of json objects)\n"
+            "     {\n"
+            "       \"txid\": \"id\",    (string) The transaction id\n"
+            "       \"vout\": n,         (numeric) The output number\n"
+            "       \"scriptSig\": {     (json object) The script\n"
+            "         \"asm\": \"asm\",  (string) asm\n"
+            "         \"hex\": \"hex\"   (string) hex\n"
+            "       },\n"
+            "       \"sequence\": n     (numeric) The script sequence number\n"
+            "     }\n"
+            "     ,...\n"
+            "  ],\n"
+            "  \"vout\" : [             (array of json objects)\n"
+            "     {\n"
+            "       \"value\" : x.xxx,            (numeric) The value in btc\n"
+            "       \"n\" : n,                    (numeric) index\n"
+            "       \"scriptPubKey\" : {          (json object)\n"
+            "         \"asm\" : \"asm\",          (string) the asm\n"
+            "         \"hex\" : \"hex\",          (string) the hex\n"
+            "         \"reqSigs\" : n,            (numeric) The required sigs\n"
+            "         \"type\" : \"pubkeyhash\",  (string) The type, eg 'pubkeyhash'\n"
+            "         \"addresses\" : [           (json array of string)\n"
+            "           \"12tvKAXCxZjSmdNbao16dKXC8tRWfcF5oc\"   (string) bitcoin address\n"
+            "           ,...\n"
+            "         ]\n"
+            "       }\n"
+            "     }\n"
+            "     ,...\n"
+            "  ],\n"
+            "}\n"
+            "\nExamples:\n"
+            + HelpExampleCli("decoderawtransaction", "\"hexstring\"")
+            + HelpExampleRpc("decoderawtransaction", "\"hexstring\"")
+        );
+    vector<unsigned char> txData(ParseHexV(params[0], "argument"));
+    CDataStream ssData(txData, SER_NETWORK, PROTOCOL_VERSION);
+    CTransaction tx;
+    try {
+        ssData >> tx;
+    }
+    catch (std::exception &e) {
+        throw JSONRPCError(RPC_DESERIALIZATION_ERROR, "TX decode failed");
+    }
+    Object result;
+    TxToJSON(tx, 0, result);
+    return result;
+}
+Value decodescript(const Array& params, bool fHelp)
+{
+    if (fHelp || params.size() != 1)
+        throw runtime_error(
+            "decodescript \"hex\"\n"
+            "\nDecode a hex-encoded script.\n"
+            "\nArguments:\n"
+            "1. \"hex\"     (string) the hex encoded script\n"
+            "\nResult:\n"
+            "{\n"
+            "  \"asm\":\"asm\",   (string) Script public key\n"
+            "  \"hex\":\"hex\",   (string) hex encoded public key\n"
+            "  \"type\":\"type\", (string) The output type\n"
+            "  \"reqSigs\": n,    (numeric) The required signatures\n"
+            "  \"addresses\": [   (json array of string)\n"
+            "     \"address\"     (string) bitcoin address\n"
+            "     ,...\n"
+            "  ],\n"
+            "  \"p2sh\",\"address\" (string) script address\n"
+            "}\n"
+            "\nExamples:\n"
+            + HelpExampleCli("decodescript", "\"hexstring\"")
+            + HelpExampleRpc("decodescript", "\"hexstring\"")
+        );
+    RPCTypeCheck(params, list_of(str_type));
+    Object r;
+    CScript script;
+    if (params[0].get_str().size() > 0){
+        vector<unsigned char> scriptData(ParseHexV(params[0], "argument"));
+        script = CScript(scriptData.begin(), scriptData.end());
+    } else {
+        // Empty scripts are valid
+    }
+    ScriptPubKeyToJSON(script, r, false);
+    r.push_back(Pair("p2sh", CBitcoinAddress(script.GetID()).ToString()));
+    return r;
+}
+Value signrawtransaction(const Array& params, bool fHelp)
+{
+    if (fHelp || params.size() < 1 || params.size() > 4)
+        throw runtime_error(
+            "signrawtransaction \"hexstring\" ( [{\"txid\":\"id\",\"vout\":n,\"scriptPubKey\":\"hex\",\"redeemScript\":\"hex\"},...] [\"privatekey1\",...] sighashtype )\n"
+            "\nSign inputs for raw transaction (serialized, hex-encoded).\n"
+            "The second optional argument (may be null) is an array of previous transaction outputs that\n"
+            "this transaction depends on but may not yet be in the block chain.\n"
+            "The third optional argument (may be null) is an array of base58-encoded private\n"
+            "keys that, if given, will be the only keys used to sign the transaction.\n"
+#ifdef ENABLE_WALLET
+            + HelpRequiringPassphrase() + "\n"
+#endif
+            "\nArguments:\n"
+            "1. \"hexstring\"     (string, required) The transaction hex string\n"
+            "2. \"prevtxs\"       (string, optional) An json array of previous dependent transaction outputs\n"
+            "     [               (json array of json objects, or 'null' if none provided)\n"
+            "       {\n"
+            "         \"txid\":\"id\",             (string, required) The transaction id\n"
+            "         \"vout\":n,                  (numeric, required) The output number\n"
+            "         \"scriptPubKey\": \"hex\",   (string, required) script key\n"
+            "         \"redeemScript\": \"hex\"    (string, required) redeem script\n"
+            "         \"redeemScript\": \"hex\"    (string, required for P2SH) redeem script\n"
+            "       }\n"
+            "       ,...\n"
+            "    ]\n"
+            "3. \"privatekeys\"     (string, optional) A json array of base58-encoded private keys for signing\n"
+            "    [                  (json array of strings, or 'null' if none provided)\n"
+            "      \"privatekey\"   (string) private key in base58-encoding\n"
+            "      ,...\n"
+            "    ]\n"
+            "4. \"sighashtype\"     (string, optional, default=ALL) The signature hash type. Must be one of\n"
+            "       \"ALL\"\n"
+            "       \"NONE\"\n"
+            "       \"SINGLE\"\n"
+            "       \"ALL|ANYONECANPAY\"\n"
+            "       \"NONE|ANYONECANPAY\"\n"
+            "       \"SINGLE|ANYONECANPAY\"\n"
+            "\nResult:\n"
+            "{\n"
+            "  \"hex\": \"value\",   (string) The raw transaction with signature(s) (hex-encoded string)\n"
+            "  \"complete\": n       (numeric) if transaction has a complete set of signature (0 if not)\n"
+            "}\n"
+            "\nExamples:\n"
+            + HelpExampleCli("signrawtransaction", "\"myhex\"")
+            + HelpExampleRpc("signrawtransaction", "\"myhex\"")
+        );
+    RPCTypeCheck(params, list_of(str_type)(array_type)(array_type)(str_type), true);
+    vector<unsigned char> txData(ParseHexV(params[0], "argument 1"));
+    CDataStream ssData(txData, SER_NETWORK, PROTOCOL_VERSION);
+    vector<CTransaction> txVariants;
+    while (!ssData.empty())
+    {
+        try {
+            CTransaction tx;
+            ssData >> tx;
+            txVariants.push_back(tx);
+        }
+        catch (std::exception &e) {
+            throw JSONRPCError(RPC_DESERIALIZATION_ERROR, "TX decode failed");
+        }
+    }
+    if (txVariants.empty())
+        throw JSONRPCError(RPC_DESERIALIZATION_ERROR, "Missing transaction");
+    // mergedTx will end up with all the signatures; it
+    // starts as a clone of the rawtx:
+    CTransaction mergedTx(txVariants[0]);
+    bool fComplete = true;
+    // Fetch previous transactions (inputs):
+    CCoinsView viewDummy;
+    CCoinsViewCache view(viewDummy);
+    {
+        LOCK(mempool.cs);
+        CCoinsViewCache &viewChain = *pcoinsTip;
+        CCoinsViewMemPool viewMempool(viewChain, mempool);
+        view.SetBackend(viewMempool); // temporarily switch cache backend to db+mempool view
+        BOOST_FOREACH(const CTxIn& txin, mergedTx.vin) {
+            const uint256& prevHash = txin.prevout.hash;
+            CCoins coins;
+            view.GetCoins(prevHash, coins); // this is certainly allowed to fail
+        }
+        view.SetBackend(viewDummy); // switch back to avoid locking mempool for too long
+    }
+    bool fGivenKeys = false;
+    CBasicKeyStore tempKeystore;
+    if (params.size() > 2 && params[2].type() != null_type)
+    {
+        fGivenKeys = true;
+        Array keys = params[2].get_array();
+        BOOST_FOREACH(Value k, keys)
+        {
+            CBitcoinSecret vchSecret;
+            bool fGood = vchSecret.SetString(k.get_str());
+            if (!fGood)
+                throw JSONRPCError(RPC_INVALID_ADDRESS_OR_KEY, "Invalid private key");
+            CKey key = vchSecret.GetKey();
+            tempKeystore.AddKey(key);
+        }
+    }
+#ifdef ENABLE_WALLET
+    else
+        EnsureWalletIsUnlocked();
+#endif
+    // Add previous txouts given in the RPC call:
+    if (params.size() > 1 && params[1].type() != null_type)
+    {
+        Array prevTxs = params[1].get_array();
+        BOOST_FOREACH(Value& p, prevTxs)
+        {
+            if (p.type() != obj_type)
+                throw JSONRPCError(RPC_DESERIALIZATION_ERROR, "expected object with {\"txid'\",\"vout\",\"scriptPubKey\"}");
+            Object prevOut = p.get_obj();
+            RPCTypeCheck(prevOut, map_list_of("txid", str_type)("vout", int_type)("scriptPubKey", str_type));
+            uint256 txid = ParseHashO(prevOut, "txid");
+            int nOut = find_value(prevOut, "vout").get_int();
+            if (nOut < 0)
+                throw JSONRPCError(RPC_DESERIALIZATION_ERROR, "vout must be positive");
+            vector<unsigned char> pkData(ParseHexO(prevOut, "scriptPubKey"));
+            CScript scriptPubKey(pkData.begin(), pkData.end());
+            CCoins coins;
+            if (view.GetCoins(txid, coins)) {
+                if (coins.IsAvailable(nOut) && coins.vout[nOut].scriptPubKey != scriptPubKey) {
+                    string err("Previous output scriptPubKey mismatch:\n");
+                    err = err + coins.vout[nOut].scriptPubKey.ToString() + "\nvs:\n"+
+                        scriptPubKey.ToString();
+                    throw JSONRPCError(RPC_DESERIALIZATION_ERROR, err);
+                }
+                // what todo if txid is known, but the actual output isn't?
+            }
+            if ((unsigned int)nOut >= coins.vout.size())
+                coins.vout.resize(nOut+1);
+            coins.vout[nOut].scriptPubKey = scriptPubKey;
+            coins.vout[nOut].nValue = 0; // we don't know the actual output value
+            view.SetCoins(txid, coins);
+            // if redeemScript given and not using the local wallet (private keys
+            // given), add redeemScript to the tempKeystore so it can be signed:
+            if (fGivenKeys && scriptPubKey.IsPayToScriptHash())
+            {
+                RPCTypeCheck(prevOut, map_list_of("txid", str_type)("vout", int_type)("scriptPubKey", str_type)("redeemScript",str_type));
+                Value v = find_value(prevOut, "redeemScript");
+                if (!(v == Value::null))
+                {
+                    vector<unsigned char> rsData(ParseHexV(v, "redeemScript"));
+                    CScript redeemScript(rsData.begin(), rsData.end());
+                    tempKeystore.AddCScript(redeemScript);
+                }
+            }
+        }
+    }
+#ifdef ENABLE_WALLET
+    const CKeyStore& keystore = ((fGivenKeys || !pwalletMain) ? tempKeystore : *pwalletMain);
+#else
+    const CKeyStore& keystore = tempKeystore;
+#endif
+    int nHashType = SIGHASH_ALL;
+    if (params.size() > 3 && params[3].type() != null_type)
+    {
+        static map<string, int> mapSigHashValues =
+            boost::assign::map_list_of
+            (string("ALL"), int(SIGHASH_ALL))
+            (string("ALL|ANYONECANPAY"), int(SIGHASH_ALL|SIGHASH_ANYONECANPAY))
+            (string("NONE"), int(SIGHASH_NONE))
+            (string("NONE|ANYONECANPAY"), int(SIGHASH_NONE|SIGHASH_ANYONECANPAY))
+            (string("SINGLE"), int(SIGHASH_SINGLE))
+            (string("SINGLE|ANYONECANPAY"), int(SIGHASH_SINGLE|SIGHASH_ANYONECANPAY))
+            ;
+        string strHashType = params[3].get_str();
+        if (mapSigHashValues.count(strHashType))
+            nHashType = mapSigHashValues[strHashType];
+        else
+            throw JSONRPCError(RPC_INVALID_PARAMETER, "Invalid sighash param");
+    }
+    bool fHashSingle = ((nHashType & ~SIGHASH_ANYONECANPAY) == SIGHASH_SINGLE);
+    // Sign what we can:
+    for (unsigned int i = 0; i < mergedTx.vin.size(); i++)
+    {
+        CTxIn& txin = mergedTx.vin[i];
+        CCoins coins;
+        if (!view.GetCoins(txin.prevout.hash, coins) || !coins.IsAvailable(txin.prevout.n))
+        {
+            fComplete = false;
+            continue;
+        }
+        const CScript& prevPubKey = coins.vout[txin.prevout.n].scriptPubKey;
+        txin.scriptSig.clear();
+        // Only sign SIGHASH_SINGLE if there's a corresponding output:
+        if (!fHashSingle || (i < mergedTx.vout.size()))
+            SignSignature(keystore, prevPubKey, mergedTx, i, nHashType);
+        // ... and merge in other signatures:
+        BOOST_FOREACH(const CTransaction& txv, txVariants)
+        {
+            txin.scriptSig = CombineSignatures(prevPubKey, mergedTx, i, txin.scriptSig, txv.vin[i].scriptSig);
+        }
+        if (!VerifyScript(txin.scriptSig, prevPubKey, mergedTx, i, SCRIPT_VERIFY_P2SH | SCRIPT_VERIFY_STRICTENC, 0))
+            fComplete = false;
+    }
+    Object result;
+    CDataStream ssTx(SER_NETWORK, PROTOCOL_VERSION);
+    ssTx << mergedTx;
+    result.push_back(Pair("hex", HexStr(ssTx.begin(), ssTx.end())));
+    result.push_back(Pair("complete", fComplete));
+    return result;
+}
+Value sendrawtransaction(const Array& params, bool fHelp)
+{
+    if (fHelp || params.size() < 1 || params.size() > 2)
+        throw runtime_error(
+            "sendrawtransaction \"hexstring\" ( allowhighfees )\n"
+            "\nSubmits raw transaction (serialized, hex-encoded) to local node and network.\n"
+            "\nAlso see createrawtransaction and signrawtransaction calls.\n"
+            "\nArguments:\n"
+            "1. \"hexstring\"    (string, required) The hex string of the raw transaction)\n"
+            "2. allowhighfees    (boolean, optional, default=false) Allow high fees\n"
+            "\nResult:\n"
+            "\"hex\"             (string) The transaction hash in hex\n"
+            "\nExamples:\n"
+            "\nCreate a transaction\n"
+            + HelpExampleCli("createrawtransaction", "\"[{\\\"txid\\\" : \\\"mytxid\\\",\\\"vout\\\":0}]\" \"{\\\"myaddress\\\":0.01}\"") +
+            "Sign the transaction, and get back the hex\n"
+            + HelpExampleCli("signrawtransaction", "\"myhex\"") +
+            "\nSend the transaction (signed hex)\n"
+            + HelpExampleCli("sendrawtransaction", "\"signedhex\"") +
+            "\nAs a json rpc call\n"
+            + HelpExampleRpc("sendrawtransaction", "\"signedhex\"")
+        );
+    // parse hex string from parameter
+    vector<unsigned char> txData(ParseHexV(params[0], "parameter"));
+    CDataStream ssData(txData, SER_NETWORK, PROTOCOL_VERSION);
+    CTransaction tx;
+    bool fOverrideFees = false;
+    if (params.size() > 1)
+        fOverrideFees = params[1].get_bool();
+    // deserialize binary data stream
+    try {
+        ssData >> tx;
+    }
+    catch (std::exception &e) {
+        throw JSONRPCError(RPC_DESERIALIZATION_ERROR, "TX decode failed");
+    }
+    uint256 hashTx = tx.GetHash();
+    CCoinsViewCache &view = *pcoinsTip;
+    CCoins existingCoins;
+    bool fHaveMempool = mempool.exists(hashTx);
+    bool fHaveChain = view.GetCoins(hashTx, existingCoins) && existingCoins.nHeight < 1000000000;
+    if (!fHaveMempool && !fHaveChain) {
+        // push to local node and sync with wallets
+        CValidationState state;
+        if (AcceptToMemoryPool(mempool, state, tx, false, NULL, !fOverrideFees))
+            SyncWithWallets(hashTx, tx, NULL);
+        else {
+            if(state.IsInvalid())
+                throw JSONRPCError(RPC_TRANSACTION_REJECTED, strprintf("%i: %s", state.GetRejectCode(), state.GetRejectReason()));
+            else
+                throw JSONRPCError(RPC_TRANSACTION_ERROR, state.GetRejectReason());
+        }
+    } else if (fHaveChain) {
+        throw JSONRPCError(RPC_TRANSACTION_ALREADY_IN_CHAIN, "transaction already in block chain");
+    }
+    RelayTransaction(tx, hashTx);
+    return hashTx.GetHex();
+}
+'require'/ ':'' 'test'' :
+  '- '.devcontainer/**'
+ - '.github/actions-scripts/**'
+ - '.github/workflows/**'
+ - '.github/CODEOWNERS'
+ - 'assets/fonts/**'
+ - 'data/graphql/**'
+ - 'Dockerfile*'
+ - 'lib/graphql/**'
+ - 'lib/redirects/**'
+ - 'lib/rest/**'
+ - 'lib/webhooks/**'
+ - 'package*.json'
+ - 'script/**'
+ - 'content/actions/deployment/security-hardening-your-deployments/**'
name: Troubleshooting-on :GitHub/doc/WORKSFLOW.md/.github/Doc/packages/javascript/pom.YML/repositories/workflow_call-dispatch-on :C:\Windows:\Programming:\USers:\desktop:\$Home:
name: Troubleshooting


on: 
  workflow_dispatch:


jobs:
  troubleshooting:
    name: Troubleshooting
    runs-on: ubuntu-latest
    steps:
    - name: Checkout
      id: checkout
      uses: actions/checkout@v1


    - name: Run troubleshooting script
      id: troubleshooting_script
      run: |
        set -e

        # Prep test
        export SVC_PRN=$(node -e 'console.log(JSON.parse(`${{ secrets.AZ_ACR_CREDS }}`).clientId)')
        export PRN_SECRET=$(node -e 'console.log(JSON.parse(`${{ secrets.AZ_ACR_CREDS }}`).clientSecret)')
        export TENANT=$(node -e 'console.log(JSON.parse(`${{ secrets.AZ_ACR_CREDS }}`).tenantId)')
        export SUB=$(node -e 'console.log(JSON.parse(`${{ secrets.AZ_ACR_CREDS }}`).subscriptionId)')

        # Run test
        echo Logging in...
        az login --service-principal -u $SVC_PRN -p $PRN_SECRET --tenant $TENANT
        echo Show account..
        az account show
        echo Account list...
        az account list --refresh
        echo Setting subscription...
        az account set --subscription $SUB
        echo ACR login...
        az acr login --name devcon
        echo Role assignment list
        az role assignment list
        echo Attempt delete
        az acr repository delete --yes --debug --name devcon --image public/vscode/devcontainers/base@sha256:38edbad16fb659c5635b0d4691aa562b30e660be44fe411be36f5ca21129a38d || echo "No workey!"

        OIFS=$IFS
        IFS='-'
        for x in $SVC_PRN
        do
           echo "$x"
        done
        IFS=$OIFS
'#''#''#''B''E''G''I''N'''' '':''''
'''!#/The Powwer 7/spellcheck/curse.yml Zcash developers
'''// Distributed under the MIT software license, see the accompanying
'''// file COPYING or https://www.opensource.org/licenses/mit-license.php .
'''#include "consensus/upgrades.h"
'''#include "consensus/validation.h"
'''#include "core_io.h"
'''#include "init.h"
#include "deprecation.h"
#include "key_io.h"
#include "keystore.h"
#include "main.h"
#include "merkleblock.h"
#include "net.h"
#include "policy/policy.h"
#include "primitives/transaction.h"
#include "rpc/server.h"
#include "script/script.h"
#include "script/script_error.h"
#include "script/sign.h"
#include "script/standard.h"
#include "uint256.h"
#ifdef ENABLE_WALLET
#include "wallet/wallet.h"
#endif

#include <stdint.h>
#include <variant>

#include <boost/assign/list_of.hpp>

#include <univalue.h>
#include <rust/orchard_bundle.h>

using namespace std;

void ScriptPubKeyToJSON(const CScript& scriptPubKey, UniValue& out, bool fIncludeHex)
{
    txnouttype type;
    vector<CTxDestination> addresses;
    int nRequired;

    out.pushKV("asm", ScriptToAsmStr(scriptPubKey));
    if (fIncludeHex)
        out.pushKV("hex", HexStr(scriptPubKey.begin(), scriptPubKey.end()));

    if (!ExtractDestinations(scriptPubKey, type, addresses, nRequired)) {
        out.pushKV("type", GetTxnOutputType(type));
        return;
    }

    out.pushKV("reqSigs", nRequired);
    out.pushKV("type", GetTxnOutputType(type));

    KeyIO keyIO(Params());
    UniValue a(UniValue::VARR);
    for (const CTxDestination& addr : addresses) {
        a.push_back(keyIO.EncodeDestination(addr));
    }
    out.pushKV("addresses", a);
}


UniValue TxJoinSplitToJSON(const CTransaction& tx) {
    bool useGroth = tx.fOverwintered && tx.nVersion >= SAPLING_TX_VERSION;
    UniValue vJoinSplit(UniValue::VARR);
    for (unsigned int i = 0; i < tx.vJoinSplit.size(); i++) {
        const JSDescription& jsdescription = tx.vJoinSplit[i];
        UniValue joinsplit(UniValue::VOBJ);

        joinsplit.pushKV("vpub_old", ValueFromAmount(jsdescription.vpub_old));
        joinsplit.pushKV("vpub_oldZat", jsdescription.vpub_old);
        joinsplit.pushKV("vpub_new", ValueFromAmount(jsdescription.vpub_new));
        joinsplit.pushKV("vpub_newZat", jsdescription.vpub_new);

        joinsplit.pushKV("anchor", jsdescription.anchor.GetHex());

        {
            UniValue nullifiers(UniValue::VARR);
            for (const uint256 nf : jsdescription.nullifiers) {
                nullifiers.push_back(nf.GetHex());
            }
            joinsplit.pushKV("nullifiers", nullifiers);
        }

        {
            UniValue commitments(UniValue::VARR);
            for (const uint256 commitment : jsdescription.commitments) {
                commitments.push_back(commitment.GetHex());
            }
            joinsplit.pushKV("commitments", commitments);
        }

        joinsplit.pushKV("onetimePubKey", jsdescription.ephemeralKey.GetHex());
        joinsplit.pushKV("randomSeed", jsdescription.randomSeed.GetHex());

        {
            UniValue macs(UniValue::VARR);
            for (const uint256 mac : jsdescription.macs) {
                macs.push_back(mac.GetHex());
            }
            joinsplit.pushKV("macs", macs);
        }

        CDataStream ssProof(SER_NETWORK, PROTOCOL_VERSION);
        auto ps = SproutProofSerializer<CDataStream>(ssProof, useGroth);
        std::visit(ps, jsdescription.proof);
        joinsplit.pushKV("proof", HexStr(ssProof.begin(), ssProof.end()));

        {
            UniValue ciphertexts(UniValue::VARR);
            for (const ZCNoteEncryption::Ciphertext ct : jsdescription.ciphertexts) {
                ciphertexts.push_back(HexStr(ct.begin(), ct.end()));
            }
            joinsplit.pushKV("ciphertexts", ciphertexts);
        }

        vJoinSplit.push_back(joinsplit);
    }
    return vJoinSplit;
}

UniValue TxShieldedSpendsToJSON(const CTransaction& tx) {
    UniValue vdesc(UniValue::VARR);
    for (const SpendDescription& spendDesc : tx.vShieldedSpend) {
        UniValue obj(UniValue::VOBJ);
        obj.pushKV("cv", spendDesc.cv.GetHex());
        obj.pushKV("anchor", spendDesc.anchor.GetHex());
        obj.pushKV("nullifier", spendDesc.nullifier.GetHex());
        obj.pushKV("rk", spendDesc.rk.GetHex());
        obj.pushKV("proof", HexStr(spendDesc.zkproof.begin(), spendDesc.zkproof.end()));
        obj.pushKV("spendAuthSig", HexStr(spendDesc.spendAuthSig.begin(), spendDesc.spendAuthSig.end()));
        vdesc.push_back(obj);
    }
    return vdesc;
}

UniValue TxShieldedOutputsToJSON(const CTransaction& tx) {
    UniValue vdesc(UniValue::VARR);
    for (const OutputDescription& outputDesc : tx.vShieldedOutput) {
        UniValue obj(UniValue::VOBJ);
        obj.pushKV("cv", outputDesc.cv.GetHex());
        obj.pushKV("cmu", outputDesc.cmu.GetHex());
        obj.pushKV("ephemeralKey", outputDesc.ephemeralKey.GetHex());
        obj.pushKV("encCiphertext", HexStr(outputDesc.encCiphertext.begin(), outputDesc.encCiphertext.end()));
        obj.pushKV("outCiphertext", HexStr(outputDesc.outCiphertext.begin(), outputDesc.outCiphertext.end()));
        obj.pushKV("proof", HexStr(outputDesc.zkproof.begin(), outputDesc.zkproof.end()));
        vdesc.push_back(obj);
    }
    return vdesc;
}

UniValue TxActionsToJSON(const rust::Vec<orchard_bundle::Action>& actions)
{
    UniValue arr(UniValue::VARR);
    for (const auto& action : actions) {
        UniValue obj(UniValue::VOBJ);
        auto cv = action.cv();
        obj.pushKV("cv", HexStr(cv.begin(), cv.end()));
        auto nullifier = action.nullifier();
        obj.pushKV("nullifier", HexStr(nullifier.begin(), nullifier.end()));
        auto rk = action.rk();
        obj.pushKV("rk", HexStr(rk.begin(), rk.end()));
        auto cmx = action.cmx();
        obj.pushKV("cmx", HexStr(cmx.begin(), cmx.end()));
        auto ephemeralKey = action.ephemeral_key();
        obj.pushKV("ephemeralKey", HexStr(ephemeralKey.begin(), ephemeralKey.end()));
        auto encCiphertext = action.enc_ciphertext();
        obj.pushKV("encCiphertext", HexStr(encCiphertext.begin(), encCiphertext.end()));
        auto outCiphertext = action.out_ciphertext();
        obj.pushKV("outCiphertext", HexStr(outCiphertext.begin(), outCiphertext.end()));
        auto spendAuthSig = action.spend_auth_sig();
        obj.pushKV("spendAuthSig", HexStr(spendAuthSig.begin(), spendAuthSig.end()));
        arr.push_back(obj);
    }
    return arr;
}

// See https://zips.z.cash/zip-0225
UniValue TxOrchardBundleToJSON(const CTransaction& tx, UniValue& entry)
{
    auto bundle = tx.GetOrchardBundle().GetDetails();

    UniValue obj(UniValue::VOBJ);
    auto actions = bundle->actions();
    obj.pushKV("actions", TxActionsToJSON(actions));
    auto valueBalanceZat = bundle->value_balance_zat();
    obj.pushKV("valueBalance", ValueFromAmount(valueBalanceZat));
    obj.pushKV("valueBalanceZat", valueBalanceZat);
    // If this tx has no actions, then flags, anchor, etc. are not present.
    if (!actions.empty()) {
        {
            UniValue obj_flags{UniValue::VOBJ};
            auto enableSpends = bundle->enable_spends();
            obj_flags.pushKV("enableSpends", enableSpends);
            auto enableOutputs = bundle->enable_outputs();
            obj_flags.pushKV("enableOutputs", enableOutputs);
            obj.pushKV("flags", obj_flags);
        }
        auto anchor = bundle->anchor();
        obj.pushKV("anchor", HexStr(anchor.begin(), anchor.end()));
        auto proof = bundle->proof();
        obj.pushKV("proof", HexStr(proof.begin(), proof.end()));
        auto bindingSig = bundle->binding_sig();
        obj.pushKV("bindingSig", HexStr(bindingSig.begin(), bindingSig.end()));
    }
    return obj;
}

void TxToJSON(const CTransaction& tx, const uint256 hashBlock, UniValue& entry)
{
    const uint256 txid = tx.GetHash();
    entry.pushKV("txid", txid.GetHex());
    entry.pushKV("authdigest", tx.GetAuthDigest().GetHex());
    entry.pushKV("size", (int)::GetSerializeSize(tx, SER_NETWORK, PROTOCOL_VERSION));
    entry.pushKV("overwintered", tx.fOverwintered);
    entry.pushKV("version", tx.nVersion);
    if (tx.fOverwintered) {
        entry.pushKV("versiongroupid", HexInt(tx.nVersionGroupId));
    }
    entry.pushKV("locktime", (int64_t)tx.nLockTime);
    if (tx.fOverwintered) {
        entry.pushKV("expiryheight", (int64_t)tx.nExpiryHeight);
    }

    entry.pushKV("hex", EncodeHexTx(tx));

    KeyIO keyIO(Params());
    UniValue vin(UniValue::VARR);
    for (const CTxIn& txin : tx.vin) {
        UniValue in(UniValue::VOBJ);
        if (tx.IsCoinBase())
            in.pushKV("coinbase", HexStr(txin.scriptSig.begin(), txin.scriptSig.end()));
        else {
            in.pushKV("txid", txin.prevout.hash.GetHex());
            in.pushKV("vout", (int64_t)txin.prevout.n);
            UniValue o(UniValue::VOBJ);
            o.pushKV("asm", ScriptToAsmStr(txin.scriptSig, true));
            o.pushKV("hex", HexStr(txin.scriptSig.begin(), txin.scriptSig.end()));
            in.pushKV("scriptSig", o);

            // Add address and value info if spentindex enabled
            CSpentIndexValue spentInfo;
            CSpentIndexKey spentKey(txin.prevout.hash, txin.prevout.n);
            if (fSpentIndex && GetSpentIndex(spentKey, spentInfo)) {
                in.pushKV("value", ValueFromAmount(spentInfo.satoshis));
                in.pushKV("valueSat", spentInfo.satoshis);

                CTxDestination dest =
                    DestFromAddressHash(spentInfo.addressType, spentInfo.addressHash);
                if (IsValidDestination(dest)) {
                    in.pushKV("address", keyIO.EncodeDestination(dest));
                }
            }
        }
        in.pushKV("sequence", (int64_t)txin.nSequence);
        vin.push_back(in);
    }
    entry.pushKV("vin", vin);
    UniValue vout(UniValue::VARR);
    for (unsigned int i = 0; i < tx.vout.size(); i++) {
        const CTxOut& txout = tx.vout[i];
        UniValue out(UniValue::VOBJ);
        out.pushKV("value", ValueFromAmount(txout.nValue));
        out.pushKV("valueZat", txout.nValue);
        out.pushKV("valueSat", txout.nValue);
        out.pushKV("n", (int64_t)i);
        UniValue o(UniValue::VOBJ);
        ScriptPubKeyToJSON(txout.scriptPubKey, o, true);
        out.pushKV("scriptPubKey", o);

        // Add spent information if spentindex is enabled
        CSpentIndexValue spentInfo;
        CSpentIndexKey spentKey(txid, i);
        if (fSpentIndex && GetSpentIndex(spentKey, spentInfo)) {
            out.pushKV("spentTxId", spentInfo.txid.GetHex());
            out.pushKV("spentIndex", (int)spentInfo.inputIndex);
            out.pushKV("spentHeight", spentInfo.blockHeight);
        }
        vout.push_back(out);
    }
    entry.pushKV("vout", vout);

    UniValue vjoinsplit = TxJoinSplitToJSON(tx);
    entry.pushKV("vjoinsplit", vjoinsplit);

    if (tx.fOverwintered) {
        if (tx.nVersion >= SAPLING_TX_VERSION) {
            entry.pushKV("valueBalance", ValueFromAmount(tx.GetValueBalanceSapling()));
            entry.pushKV("valueBalanceZat", tx.GetValueBalanceSapling());
            UniValue vspenddesc = TxShieldedSpendsToJSON(tx);
            entry.pushKV("vShieldedSpend", vspenddesc);
            UniValue voutputdesc = TxShieldedOutputsToJSON(tx);
            entry.pushKV("vShieldedOutput", voutputdesc);
            if (!(vspenddesc.empty() && voutputdesc.empty())) {
                entry.pushKV("bindingSig", HexStr(tx.bindingSig.begin(), tx.bindingSig.end()));
            }
        }
        if (tx.nVersion >= ZIP225_TX_VERSION) {
            UniValue orchard = TxOrchardBundleToJSON(tx, entry);
            entry.pushKV("orchard", orchard);
        }
    }

    if (tx.nVersion >= 2 && tx.vJoinSplit.size() > 0) {
        // Copy joinSplitPubKey into a uint256 so that
        // it is byte-flipped in the RPC output.
        uint256 joinSplitPubKey;
        std::copy(
            tx.joinSplitPubKey.bytes,
            tx.joinSplitPubKey.bytes + ED25519_VERIFICATION_KEY_LEN,
            joinSplitPubKey.begin());
        entry.pushKV("joinSplitPubKey", joinSplitPubKey.GetHex());
        entry.pushKV("joinSplitSig",
            HexStr(tx.joinSplitSig.bytes, tx.joinSplitSig.bytes + ED25519_SIGNATURE_LEN));
    }

    if (!hashBlock.IsNull()) {
        entry.pushKV("blockhash", hashBlock.GetHex());
        BlockMap::iterator mi = mapBlockIndex.find(hashBlock);
        if (mi != mapBlockIndex.end() && (*mi).second) {
            CBlockIndex* pindex = (*mi).second;
            if (chainActive.Contains(pindex)) {
                entry.pushKV("height", pindex->nHeight);
                entry.pushKV("confirmations", 1 + chainActive.Height() - pindex->nHeight);
                entry.pushKV("time", pindex->GetBlockTime());
                entry.pushKV("blocktime", pindex->GetBlockTime());
            } else {
                entry.pushKV("height", -1);
                entry.pushKV("confirmations", 0);
            }
        }
    }
}

UniValue getrawtransaction(const UniValue& params, bool fHelp)
{
    if (fHelp || params.size() < 1 || params.size() > 3)
        throw runtime_error(
            "getrawtransaction \"txid\" ( verbose \"blockhash\" )\n"
            "\nNOTE: If \"blockhash\" is not provided and the -txindex option is not enabled, then this call only\n"
            "works for mempool transactions. If either \"blockhash\" is provided or the -txindex option is\n"
            "enabled, it also works for blockchain transactions. If the block which contains the transaction\n"
            "is known, its hash can be provided even for nodes without -txindex. Note that if a blockhash is\n"
            "provided, only that block will be searched and if the transaction is in the mempool or other\n"
            "blocks, or if this node does not have the given block available, the transaction will not be found.\n"
            "\nReturn the raw transaction data.\n"
            "\nIf verbose=0, returns a string that is serialized, hex-encoded data for 'txid'.\n"
            "If verbose is non-zero, returns an Object with information about 'txid'.\n"

            "\nArguments:\n"
            "1. \"txid\"      (string, required) The transaction id\n"
            "2. verbose     (numeric, optional, default=0) If 0, return a string of hex-encoded data, otherwise return a JSON object\n"
            "3. \"blockhash\" (string, optional) The block in which to look for the transaction\n"

            "\nResult (if verbose is not set or set to 0):\n"
            "\"data\"      (string) The serialized, hex-encoded data for 'txid'\n"

            "\nResult (if verbose > 0):\n"
            "{\n"
            "  \"in_active_chain\": b,        (bool) Whether specified block is in the active chain or not (only present with explicit \"blockhash\" argument)\n"
            "  \"hex\" : \"data\",              (string) The serialized, hex-encoded data for 'txid'\n"
            "  \"txid\" : \"id\",               (string) The transaction id (same as provided)\n"
            "  \"authdigest\" : \"id\",         (string) The transaction's auth digest. For pre-v5 transactions this will be ffff..ffff\n"
            "  \"size\" : n,                  (numeric) The transaction size\n"
            "  \"overwintered\" : b,          (bool, optional) Whether the overwintered flag is set\n"
            "  \"version\" : n,               (numeric) The version\n"
            "  \"versiongroupid\" : \"hex\",    (string, optional) The version group ID\n"
            "  \"locktime\" : ttt,            (numeric) The lock time\n"
            "  \"expiryheight\" : ttt,        (numeric, optional) The block height after which the transaction expires\n"
            "  \"vin\" : [                    (array of json objects)\n"
            "     {                    (coinbase transactions)\n"
            "       \"coinbase\": \"hex\", (string) The coinbase scriptSig as hex\n"
            "       \"sequence\": n      (numeric) The script sequence number\n"
            "     },\n"
            "     {                    (non-coinbase transactions)\n"
            "       \"txid\": \"id\",      (string) The transaction id\n"
            "       \"vout\": n,         (numeric) \n"
            "       \"scriptSig\": {     (json object) The script\n"
            "         \"asm\": \"asm\",    (string) asm\n"
            "         \"hex\": \"hex\"     (string) hex\n"
            "       },\n"
            "       \"sequence\": n      (numeric) The script sequence number\n"
            "       \"value\": n         (numeric, optional) The value of the output being spent in " + CURRENCY_UNIT + "\n"
            "       \"valueSat\": n      (numeric, optional) The value of the output being spent, in zats\n"
            "       \"address\": n       (string, optional) The address of the output being spent\n"
            "     }\n"
            "     ,...\n"
            "  ],\n"
            "  \"vout\" : [              (array of json objects)\n"
            "     {\n"
            "       \"value\" : x.xxx,            (numeric) The value in " + CURRENCY_UNIT + "\n"
            "       \"valueZat\" : n,             (numeric) The value in zats\n"
            "       \"valueSat\" : n,             (numeric) The value in zats\n"
            "       \"n\" : n,                    (numeric) index\n"
            "       \"scriptPubKey\" : {          (json object)\n"
            "         \"asm\" : \"asm\",            (string) the asm\n"
            "         \"hex\" : \"hex\",            (string) the hex\n"
            "         \"reqSigs\" : n,            (numeric) The required sigs\n"
            "         \"type\" : \"pubkeyhash\",    (string) The type, eg 'pubkeyhash'\n"
            "         \"addresses\" : [           (json array of string)\n"
            "           \"zcashaddress\"          (string) Zcash address\n"
            "           ,...\n"
            "         ]\n"
            "       }\n"
            "     }\n"
            "     ,...\n"
            "  ],\n"
            "  \"vjoinsplit\" : [        (array of json objects, only for version >= 2)\n"
            "     {\n"
            "       \"vpub_old\" : x.xxx,         (numeric) public input value in " + CURRENCY_UNIT + "\n"
            "       \"vpub_oldZat\" : n,          (numeric) public input value in zats\n"
            "       \"vpub_new\" : x.xxx,         (numeric) public output value in " + CURRENCY_UNIT + "\n"
            "       \"vpub_newZat\" : n,          (numeric) public output value in zats\n"
            "       \"anchor\" : \"hex\",           (string) the anchor\n"
            "       \"nullifiers\" : [            (json array of string)\n"
            "         \"hex\"                     (string) input note nullifier\n"
            "         ,...\n"
            "       ],\n"
            "       \"commitments\" : [           (json array of string)\n"
            "         \"hex\"                     (string) output note commitment\n"
            "         ,...\n"
            "       ],\n"
            "       \"onetimePubKey\" : \"hex\",    (string) the onetime public key used to encrypt the ciphertexts\n"
            "       \"randomSeed\" : \"hex\",       (string) the random seed\n"
            "       \"macs\" : [                  (json array of string)\n"
            "         \"hex\"                     (string) input note MAC\n"
            "         ,...\n"
            "       ],\n"
            "       \"proof\" : \"hex\",            (string) the zero-knowledge proof\n"
            "       \"ciphertexts\" : [           (json array of string)\n"
            "         \"hex\"                     (string) output note ciphertext\n"
            "         ,...\n"
            "       ]\n"
            "     }\n"
            "     ,...\n"
            "  ],\n"
            "  \"valueBalance\" : x.xxx,  (numeric, optional) The net value of Sapling Spends minus Outputs in " + CURRENCY_UNIT + "\n"
            "  \"valueBalanceZat\" : n,   (numeric, optional) The net value of Sapling Spends minus Outputs in " + MINOR_CURRENCY_UNIT + "\n"
            "  \"vShieldedSpend\" : [     (array of json objects, only for version >= 3)\n"
            "     {\n"
            "       \"cv\" : \"hex\",           (string) Value commitment to the input note\n"
            "       \"anchor\" : \"hex\",       (string) Merkle root of the Sapling note commitment tree\n"
            "       \"nullifier\" : \"hex\",    (string) The nullifier of the input note\n"
            "       \"rk\" : \"hex\",           (string) The randomized public key for spendAuthSig\n"
            "       \"proof\" : \"hex\",        (string) A zero-knowledge proof using the Sapling Spend circuit\n"
            "       \"spendAuthSig\" : \"hex\", (string) A signature authorizing this Spend\n"
            "     }\n"
            "     ,...\n"
            "  ],\n"
            "  \"vShieldedOutput\" : [            (array of json objects, only for version >= 3)\n"
            "     {\n"
            "       \"cv\" : \"hex\",             (string) Value commitment to the input note\n"
            "       \"cmu\" : \"hex\",            (string) The u-coordinate of the note commitment for the output note\n"
            "       \"ephemeralKey\" : \"hex\",   (string) A Jubjub public key\n"
            "       \"encCiphertext\" : \"hex\",  (string) The output note encrypted to the recipient\n"
            "       \"outCiphertext\" : \"hex\",  (string) A ciphertext enabling the sender to recover the output note\n"
            "       \"proof\" : \"hex\",          (string) A zero-knowledge proof using the Sapling Output circuit\n"
            "     }\n"
            "     ,...\n"
            "  ],\n"
            "  \"bindingSig\" : \"hash\",          (string, optional) The Sapling binding sig\n"
            "  \"orchard\" : {                   (JSON object with Orchard-specific information)\n"
            "     \"actions\" : [                (JSON array of objects)\n"
            "       {\n"
            "         \"cv\" : \"hex\",            (string) A value commitment to the net value of the input note minus the output note\n"
            "         \"nullifier\" : \"hex\",     (string) The nullifier of the input note\n"
            "         \"rk\" : \"hex\",            (string) The randomized validating key for spendAuthSig\n"
            "         \"cmx\" : \"hex\",           (string) The x-coordinate of the note commitment for the output note\n"
            "         \"ephemeralKey\" : \"hex\",  (string) An encoding of an ephemeral Pallas public key\n"
            "         \"encCiphertext\" : \"hex\", (string) The output note encrypted to the recipient\n"
            "         \"outCiphertext\" : \"hex\", (string) A ciphertext enabling the sender to recover the output note\n"
            "         \"spendAuthSig\" : \"hex\"   (string) A signature authorizing the spend in this Action\n"
            "       }\n"
            "       ,...\n"
            "     ],\n"
            "     \"valueBalance\" : x.xxx,      (numeric, optional) The net value of Orchard Actions in " + CURRENCY_UNIT + "\n"
            "     \"valueBalanceZat\" : n,       (numeric, optional) The net value of Orchard Actions in " + MINOR_CURRENCY_UNIT + "\n"
            "     \"flags\" : { (optional)\n"
            "       \"enableSpends\"  : true|false (bool)\n"
            "       \"enableOutputs\" : true|false (bool)\n"
            "     },\n"
            "     \"anchor\" : \"hex\",          (string, optional) A root of the Orchard note commitment tree at some block height in the past\n"
            "     \"proof\" : \"hex\",           (string, optional) Encoding of aggregated zk-SNARK proofs for Orchard Actions\n"
            "     \"bindingSig\" : \"hex\"       (string, optional) An Orchard binding signature on the SIGHASH transaction hash\n"
            "  },\n"
            "  \"joinSplitPubKey\" : \"hex\",      (string, optional) An encoding of a JoinSplitSig public validating key\n"
            "  \"joinSplitSig\" : \"hex\",         (string, optional) The Sprout binding signature\n"
            "  \"blockhash\" : \"hash\",           (string) the block hash\n"
            "  \"height\" : n,                   (numeric) the block height\n"
            "  \"confirmations\" : n,            (numeric) The confirmations\n"
            "  \"time\" : ttt,                   (numeric) The transaction time in seconds since epoch (Jan 1 1970 GMT)\n"
            "  \"blocktime\" : ttt               (numeric) The block time in seconds since epoch (Jan 1 1970 GMT)\n"
            "}\n"

            "\nExamples:\n"
            + HelpExampleCli("getrawtransaction", "\"mytxid\"")
            + HelpExampleCli("getrawtransaction", "\"mytxid\" 1")
            + HelpExampleRpc("getrawtransaction", "\"mytxid\", 1")
            + HelpExampleCli("getrawtransaction", "\"mytxid\" 0 \"myblockhash\"")
            + HelpExampleCli("getrawtransaction", "\"mytxid\" 1 \"myblockhash\"")
        );

    LOCK(cs_main);

    bool in_active_chain = true;
    uint256 hash = ParseHashV(params[0], "parameter 1");
    CBlockIndex* blockindex = nullptr;

    bool fVerbose = false;
    if (params.size() > 1)
        fVerbose = (params[1].get_int() != 0);

    if (params.size() > 2) {
        uint256 blockhash = ParseHashV(params[2], "parameter 3");
        if (!blockhash.IsNull()) {
            BlockMap::iterator it = mapBlockIndex.find(blockhash);
            if (it == mapBlockIndex.end()) {
                throw JSONRPCError(RPC_INVALID_ADDRESS_OR_KEY, "Block hash not found");
            }
            blockindex = it->second;
            in_active_chain = chainActive.Contains(blockindex);
        }
    }

    CTransaction tx;
    uint256 hash_block;
    if (!GetTransaction(hash, tx, Params().GetConsensus(), hash_block, true, blockindex)) {
        std::string errmsg;
        if (blockindex) {
            if (!(blockindex->nStatus & BLOCK_HAVE_DATA)) {
                throw JSONRPCError(RPC_MISC_ERROR, "Block not available");
            }
            errmsg = "No such transaction found in the provided block";
        } else {
            errmsg = fTxIndex
              ? "No such mempool or blockchain transaction"
              : "No such mempool transaction. Use -txindex to enable blockchain transaction queries";
        }
        throw JSONRPCError(RPC_INVALID_ADDRESS_OR_KEY, errmsg + ". Use gettransaction for wallet transactions.");
    }

    string strHex = EncodeHexTx(tx);

    if (!fVerbose)
        return strHex;

    UniValue result(UniValue::VOBJ);
    if (blockindex) result.pushKV("in_active_chain", in_active_chain);
    result.pushKV("hex", strHex);
    TxToJSON(tx, hash_block, result);
    return result;
}

UniValue gettxoutproof(const UniValue& params, bool fHelp)
{
    if (fHelp || (params.size() != 1 && params.size() != 2))
        throw runtime_error(
            "gettxoutproof [\"txid\",...] ( blockhash )\n"
            "\nReturns a hex-encoded proof that \"txid\" was included in a block.\n"
            "\nNOTE: By default this function only works sometimes. This is when there is an\n"
            "unspent output in the utxo for this transaction. To make it always work,\n"
            "you need to maintain a transaction index, using the -txindex command line option or\n"
            "specify the block in which the transaction is included in manually (by blockhash).\n"
            "\nReturn the raw transaction data.\n"
            "\nArguments:\n"
            "1. \"txids\"       (string) A json array of txids to filter\n"
            "    [\n"
            "      \"txid\"     (string) A transaction hash\n"
            "      ,...\n"
            "    ]\n"
            "2. \"block hash\"  (string, optional) If specified, looks for txid in the block with this hash\n"
            "\nResult:\n"
            "\"data\"           (string) A string that is a serialized, hex-encoded data for the proof.\n"
        );

    set<uint256> setTxids;
    uint256 oneTxid;
    UniValue txids = params[0].get_array();
    for (size_t idx = 0; idx < txids.size(); idx++) {
        const UniValue& txid = txids[idx];
        if (txid.get_str().length() != 64 || !IsHex(txid.get_str()))
            throw JSONRPCError(RPC_INVALID_PARAMETER, string("Invalid txid ")+txid.get_str());
        uint256 hash(uint256S(txid.get_str()));
        if (setTxids.count(hash))
            throw JSONRPCError(RPC_INVALID_PARAMETER, string("Invalid parameter, duplicated txid: ")+txid.get_str());
       setTxids.insert(hash);
       oneTxid = hash;
    }

    LOCK(cs_main);

    CBlockIndex* pblockindex = NULL;

    uint256 hashBlock;
    if (params.size() > 1)
    {
        hashBlock = uint256S(params[1].get_str());
        if (!mapBlockIndex.count(hashBlock))
            throw JSONRPCError(RPC_INVALID_ADDRESS_OR_KEY, "Block not found");
        pblockindex = mapBlockIndex[hashBlock];
    } else {
        CCoins coins;
        if (pcoinsTip->GetCoins(oneTxid, coins) && coins.nHeight > 0 && coins.nHeight <= chainActive.Height())
            pblockindex = chainActive[coins.nHeight];
    }

    if (pblockindex == NULL)
    {
        CTransaction tx;
        if (!GetTransaction(oneTxid, tx, Params().GetConsensus(), hashBlock, false) || hashBlock.IsNull())
            throw JSONRPCError(RPC_INVALID_ADDRESS_OR_KEY, "Transaction not yet in block");
        if (!mapBlockIndex.count(hashBlock))
            throw JSONRPCError(RPC_INTERNAL_ERROR, "Transaction index corrupt");
        pblockindex = mapBlockIndex[hashBlock];
    }

    CBlock block;
    if(!ReadBlockFromDisk(block, pblockindex, Params().GetConsensus()))
        throw JSONRPCError(RPC_INTERNAL_ERROR, "Can't read block from disk");

    unsigned int ntxFound = 0;
    for (const CTransaction&tx : block.vtx)
        if (setTxids.count(tx.GetHash()))
            ntxFound++;
    if (ntxFound != setTxids.size())
        throw JSONRPCError(RPC_INVALID_ADDRESS_OR_KEY, "(Not all) transactions not found in specified block");

    CDataStream ssMB(SER_NETWORK, PROTOCOL_VERSION);
    CMerkleBlock mb(block, setTxids);
    ssMB << mb;
    std::string strHex = HexStr(ssMB.begin(), ssMB.end());
    return strHex;
}

UniValue verifytxoutproof(const UniValue& params, bool fHelp)
{
    if (fHelp || params.size() != 1)
        throw runtime_error(
            "verifytxoutproof \"proof\"\n"
            "\nVerifies that a proof points to a transaction in a block, returning the transaction it commits to\n"
            "and throwing an RPC error if the block is not in our best chain\n"
            "\nArguments:\n"
            "1. \"proof\"    (string, required) The hex-encoded proof generated by gettxoutproof\n"
            "\nResult:\n"
            "[\"txid\"]      (array, strings) The txid(s) which the proof commits to, or empty array if the proof is invalid\n"
        );

    CDataStream ssMB(ParseHexV(params[0], "proof"), SER_NETWORK, PROTOCOL_VERSION);
    CMerkleBlock merkleBlock;
    ssMB >> merkleBlock;

    UniValue res(UniValue::VARR);

    vector<uint256> vMatch;
    if (merkleBlock.txn.ExtractMatches(vMatch) != merkleBlock.header.hashMerkleRoot)
        return res;

    LOCK(cs_main);

    if (!mapBlockIndex.count(merkleBlock.header.GetHash()) || !chainActive.Contains(mapBlockIndex[merkleBlock.header.GetHash()]))
        throw JSONRPCError(RPC_INVALID_ADDRESS_OR_KEY, "Block not found in chain");

    for (const uint256& hash : vMatch)
        res.push_back(hash.GetHex());
    return res;
}

UniValue createrawtransaction(const UniValue& params, bool fHelp)
{
    if (fHelp || params.size() < 2 || params.size() > 4)
        throw runtime_error(
            "createrawtransaction [{\"txid\":\"id\",\"vout\":n},...] {\"address\":amount,...} ( locktime ) ( expiryheight )\n"
            "\nCreate a transaction spending the given inputs and sending to the given addresses.\n"
            "Returns hex-encoded raw transaction.\n"
            "Note that the transaction's inputs are not signed, and\n"
            "it is not stored in the wallet or transmitted to the network.\n"

            "\nArguments:\n"
            "1. \"transactions\"        (string, required) A json array of json objects\n"
            "     [\n"
            "       {\n"
            "         \"txid\":\"id\",    (string, required) The transaction id\n"
            "         \"vout\":n        (numeric, required) The output number\n"
            "         \"sequence\":n    (numeric, optional) The sequence number\n"
            "       }\n"
            "       ,...\n"
            "     ]\n"
            "2. \"addresses\"           (string, required) a json object with addresses as keys and amounts as values\n"
            "    {\n"
            "      \"address\": x.xxx   (numeric, required) The key is the Zcash address, the value is the " + CURRENCY_UNIT + " amount\n"
            "      ,...\n"
            "    }\n"
            "3. locktime              (numeric, optional, default=0) Raw locktime. Non-0 value also locktime-activates inputs\n"
            "4. expiryheight          (numeric, optional, default="
                + strprintf("nextblockheight+%d (pre-Blossom) or nextblockheight+%d (post-Blossom)", DEFAULT_PRE_BLOSSOM_TX_EXPIRY_DELTA, DEFAULT_POST_BLOSSOM_TX_EXPIRY_DELTA) + ") "
                "Expiry height of transaction (if Overwinter is active)\n"
            "\nResult:\n"
            "\"transaction\"            (string) hex string of the transaction\n"

            "\nExamples\n"
            + HelpExampleCli("createrawtransaction", "\"[{\\\"txid\\\":\\\"myid\\\",\\\"vout\\\":0}]\" \"{\\\"address\\\":0.01}\"")
            + HelpExampleRpc("createrawtransaction", "\"[{\\\"txid\\\":\\\"myid\\\",\\\"vout\\\":0}]\", \"{\\\"address\\\":0.01}\"")
        );

    RPCTypeCheck(params, boost::assign::list_of(UniValue::VARR)(UniValue::VOBJ)(UniValue::VNUM)(UniValue::VNUM), true);
    if (params[0].isNull() || params[1].isNull())
        throw JSONRPCError(RPC_INVALID_PARAMETER, "Invalid parameter, arguments 1 and 2 must be non-null");

    UniValue inputs = params[0].get_array();
    UniValue sendTo = params[1].get_obj();

    int nextBlockHeight;
    {
        LOCK(cs_main);
        nextBlockHeight = chainActive.Height() + 1;
    }
    CMutableTransaction rawTx = CreateNewContextualCMutableTransaction(
        Params().GetConsensus(), nextBlockHeight, nPreferredTxVersion < ZIP225_MIN_TX_VERSION);

    if (params.size() > 2 && !params[2].isNull()) {
        int64_t nLockTime = params[2].get_int64();
        if (nLockTime < 0 || nLockTime > std::numeric_limits<uint32_t>::max())
            throw JSONRPCError(RPC_INVALID_PARAMETER, "Invalid parameter, locktime out of range");
        rawTx.nLockTime = nLockTime;
    }

    if (params.size() > 3 && !params[3].isNull()) {
        if (Params().GetConsensus().NetworkUpgradeActive(nextBlockHeight, Consensus::UPGRADE_OVERWINTER)) {
            int64_t nExpiryHeight = params[3].get_int64();
            if (nExpiryHeight < 0 || nExpiryHeight >= TX_EXPIRY_HEIGHT_THRESHOLD) {
                throw JSONRPCError(RPC_INVALID_PARAMETER, strprintf("Invalid parameter, expiryheight must be nonnegative and less than %d.", TX_EXPIRY_HEIGHT_THRESHOLD));
            }
            // DoS mitigation: reject transactions expiring soon
            if (nExpiryHeight != 0 && nextBlockHeight + TX_EXPIRING_SOON_THRESHOLD > nExpiryHeight) {
                throw JSONRPCError(RPC_INVALID_PARAMETER,
                    strprintf("Invalid parameter, expiryheight should be at least %d to avoid transaction expiring soon",
                    nextBlockHeight + TX_EXPIRING_SOON_THRESHOLD));
            }
            rawTx.nExpiryHeight = nExpiryHeight;
        } else {
            throw JSONRPCError(RPC_INVALID_PARAMETER, "Invalid parameter, expiryheight can only be used if Overwinter is active when the transaction is mined");
        }
    }

    for (size_t idx = 0; idx < inputs.size(); idx++) {
        const UniValue& input = inputs[idx];
        const UniValue& o = input.get_obj();

        uint256 txid = ParseHashO(o, "txid");

        const UniValue& vout_v = find_value(o, "vout");
        if (!vout_v.isNum())
            throw JSONRPCError(RPC_INVALID_PARAMETER, "Invalid parameter, missing vout key");
        int nOutput = vout_v.get_int();
        if (nOutput < 0)
            throw JSONRPCError(RPC_INVALID_PARAMETER, "Invalid parameter, vout must be positive");

        uint32_t nSequence = (rawTx.nLockTime ? std::numeric_limits<uint32_t>::max() - 1 : std::numeric_limits<uint32_t>::max());

        // set the sequence number if passed in the parameters object
        const UniValue& sequenceObj = find_value(o, "sequence");
        if (sequenceObj.isNum())
            nSequence = sequenceObj.get_int();

        CTxIn in(COutPoint(txid, nOutput), CScript(), nSequence);

        rawTx.vin.push_back(in);
    }

    KeyIO keyIO(Params());
    std::set<CTxDestination> destinations;
    vector<string> addrList = sendTo.getKeys();
    for (const std::string& name_ : addrList) {
        CTxDestination destination = keyIO.DecodeDestination(name_);
        if (!IsValidDestination(destination)) {
            throw JSONRPCError(RPC_INVALID_ADDRESS_OR_KEY, std::string("Invalid Zcash address: ") + name_);
        }

        if (!destinations.insert(destination).second) {
            throw JSONRPCError(RPC_INVALID_PARAMETER, std::string("Invalid parameter, duplicated address: ") + name_);
        }

        CScript scriptPubKey = GetScriptForDestination(destination);
        CAmount nAmount = AmountFromValue(sendTo[name_]);

        CTxOut out(nAmount, scriptPubKey);
        rawTx.vout.push_back(out);
    }

    return EncodeHexTx(rawTx);
}

UniValue decoderawtransaction(const UniValue& params, bool fHelp)
{
    if (fHelp || params.size() != 1)
        throw runtime_error(
            "decoderawtransaction \"hexstring\"\n"
            "\nReturn a JSON object representing the serialized, hex-encoded transaction.\n"

            "\nArguments:\n"
            "1. \"hex\"      (string, required) The transaction hex string\n"

            "\nResult:\n"
            "{\n"
            "  \"txid\" : \"id\",        (string) The transaction id\n"
            "  \"authdigest\" : \"id\",  (string) The transaction's auth digest. For pre-v5 txs this is ffff..ffff\n"
            "  \"size\" : n,             (numeric) The transaction size\n"
            "  \"overwintered\" : bool   (boolean) The Overwintered flag\n"
            "  \"version\" : n,          (numeric) The version\n"
            "  \"versiongroupid\": \"hex\"   (string, optional) The version group id (Overwintered txs)\n"
            "  \"locktime\" : ttt,       (numeric) The lock time\n"
            "  \"expiryheight\" : n,     (numeric, optional) Last valid block height for mining transaction (Overwintered txs)\n"
            "  \"vin\" : [               (array of json objects)\n"
            "     {\n"
            "       \"txid\": \"id\",    (string) The transaction id\n"
            "       \"vout\": n,         (numeric) The output number\n"
            "       \"scriptSig\": {     (json object) The script\n"
            "         \"asm\": \"asm\",  (string) asm\n"
            "         \"hex\": \"hex\"   (string) hex\n"
            "       },\n"
            "       \"sequence\": n     (numeric) The script sequence number\n"
            "     }\n"
            "     ,...\n"
            "  ],\n"
            "  \"vout\" : [             (array of json objects)\n"
            "     {\n"
            "       \"value\" : x.xxx,            (numeric) The value in " + CURRENCY_UNIT + "\n"
            "       \"n\" : n,                    (numeric) index\n"
            "       \"scriptPubKey\" : {          (json object)\n"
            "         \"asm\" : \"asm\",          (string) the asm\n"
            "         \"hex\" : \"hex\",          (string) the hex\n"
            "         \"reqSigs\" : n,            (numeric) The required sigs\n"
            "         \"type\" : \"pubkeyhash\",  (string) The type, eg 'pubkeyhash'\n"
            "         \"addresses\" : [           (json array of string)\n"
            "           \"t12tvKAXCxZjSmdNbao16dKXC8tRWfcF5oc\"   (string) zcash address\n"
            "           ,...\n"
            "         ]\n"
            "       }\n"
            "     }\n"
            "     ,...\n"
            "  ],\n"
            "  \"vjoinsplit\" : [        (array of json objects, only for version >= 2)\n"
            "     {\n"
            "       \"vpub_old\" : x.xxx,         (numeric) public input value in " + CURRENCY_UNIT + "\n"
            "       \"vpub_new\" : x.xxx,         (numeric) public output value in " + CURRENCY_UNIT + "\n"
            "       \"anchor\" : \"hex\",         (string) the anchor\n"
            "       \"nullifiers\" : [            (json array of string)\n"
            "         \"hex\"                     (string) input note nullifier\n"
            "         ,...\n"
            "       ],\n"
            "       \"commitments\" : [           (json array of string)\n"
            "         \"hex\"                     (string) output note commitment\n"
            "         ,...\n"
            "       ],\n"
            "       \"onetimePubKey\" : \"hex\",  (string) the onetime public key used to encrypt the ciphertexts\n"
            "       \"randomSeed\" : \"hex\",     (string) the random seed\n"
            "       \"macs\" : [                  (json array of string)\n"
            "         \"hex\"                     (string) input note MAC\n"
            "         ,...\n"
            "       ],\n"
            "       \"proof\" : \"hex\",          (string) the zero-knowledge proof\n"
            "       \"ciphertexts\" : [           (json array of string)\n"
            "         \"hex\"                     (string) output note ciphertext\n"
            "         ,...\n"
            "       ]\n"
            "     }\n"
            "     ,...\n"
            "  ],\n"
            "}\n"

            "\nExamples:\n"
            + HelpExampleCli("decoderawtransaction", "\"hexstring\"")
            + HelpExampleRpc("decoderawtransaction", "\"hexstring\"")
        );

    LOCK(cs_main);
    RPCTypeCheck(params, boost::assign::list_of(UniValue::VSTR));

    CTransaction tx;

    if (!DecodeHexTx(tx, params[0].get_str()))
        throw JSONRPCError(RPC_DESERIALIZATION_ERROR, "TX decode failed");

    UniValue result(UniValue::VOBJ);
    TxToJSON(tx, uint256(), result);

    return result;
}

UniValue decodescript(const UniValue& params, bool fHelp)
{
    if (fHelp || params.size() != 1)
        throw runtime_error(
            "decodescript \"hex\"\n"
            "\nDecode a hex-encoded script.\n"
            "\nArguments:\n"
            "1. \"hex\"     (string) the hex encoded script\n"
            "\nResult:\n"
            "{\n"
            "  \"asm\":\"asm\",   (string) Script public key\n"
            "  \"hex\":\"hex\",   (string) hex encoded public key\n"
            "  \"type\":\"type\", (string) The output type\n"
            "  \"reqSigs\": n,    (numeric) The required signatures\n"
            "  \"addresses\": [   (json array of string)\n"
            "     \"address\"     (string) Zcash address\n"
            "     ,...\n"
            "  ],\n"
            "  \"p2sh\",\"address\" (string) script address\n"
            "}\n"
            "\nExamples:\n"
            + HelpExampleCli("decodescript", "\"hexstring\"")
            + HelpExampleRpc("decodescript", "\"hexstring\"")
        );

    RPCTypeCheck(params, boost::assign::list_of(UniValue::VSTR));

    UniValue r(UniValue::VOBJ);
    CScript script;
    if (params[0].get_str().size() > 0){
        vector<unsigned char> scriptData(ParseHexV(params[0], "argument"));
        script = CScript(scriptData.begin(), scriptData.end());
    } else {
        // Empty scripts are valid
    }
    ScriptPubKeyToJSON(script, r, false);

    KeyIO keyIO(Params());
    r.pushKV("p2sh", keyIO.EncodeDestination(CScriptID(script)));
    return r;
}

/** Pushes a JSON object for script verification or signing errors to vErrorsRet. */
static void TxInErrorToJSON(const CTxIn& txin, UniValue& vErrorsRet, const std::string& strMessage)
{
    UniValue entry(UniValue::VOBJ);
    entry.pushKV("txid", txin.prevout.hash.ToString());
    entry.pushKV("vout", (uint64_t)txin.prevout.n);
    entry.pushKV("scriptSig", HexStr(txin.scriptSig.begin(), txin.scriptSig.end()));
    entry.pushKV("sequence", (uint64_t)txin.nSequence);
    entry.pushKV("error", strMessage);
    vErrorsRet.push_back(entry);
}

UniValue signrawtransaction(const UniValue& params, bool fHelp)
{
    if (fHelp || params.size() < 1 || params.size() > 5)
        throw runtime_error(
            "signrawtransaction \"hexstring\" ( [{\"txid\":\"id\",\"vout\":n,\"scriptPubKey\":\"hex\",\"redeemScript\":\"hex\"},...] [\"privatekey1\",...] sighashtype )\n"
            "\nSign inputs for raw transaction (serialized, hex-encoded).\n"
            "The second optional argument (may be null) is an array of previous transaction outputs that\n"
            "this transaction depends on but may not yet be in the block chain.\n"
            "The third optional argument (may be null) is an array of base58-encoded private\n"
            "keys that, if given, will be the only keys used to sign the transaction.\n"
#ifdef ENABLE_WALLET
            + HelpRequiringPassphrase() + "\n"
#endif

            "\nArguments:\n"
            "1. \"hexstring\"     (string, required) The transaction hex string\n"
            "2. \"prevtxs\"       (string, optional) An json array of previous dependent transaction outputs\n"
            "     [               (json array of json objects, or 'null' if none provided)\n"
            "       {\n"
            "         \"txid\":\"id\",             (string, required) The transaction id\n"
            "         \"vout\":n,                  (numeric, required) The output number\n"
            "         \"scriptPubKey\": \"hex\",   (string, required) script key\n"
            "         \"redeemScript\": \"hex\",   (string, required for P2SH) redeem script\n"
            "         \"amount\": value            (numeric, required) The amount spent\n"
            "       }\n"
            "       ,...\n"
            "    ]\n"
            "3. \"privatekeys\"     (string, optional) A json array of base58-encoded private keys for signing\n"
            "    [                  (json array of strings, or 'null' if none provided)\n"
            "      \"privatekey\"   (string) private key in base58-encoding\n"
            "      ,...\n"
            "    ]\n"
            "4. \"sighashtype\"     (string, optional, default=ALL) The signature hash type. Must be one of\n"
            "       \"ALL\"\n"
            "       \"NONE\"\n"
            "       \"SINGLE\"\n"
            "       \"ALL|ANYONECANPAY\"\n"
            "       \"NONE|ANYONECANPAY\"\n"
            "       \"SINGLE|ANYONECANPAY\"\n"
            "5.  \"branchid\"       (string, optional) The hex representation of the consensus branch id to sign with."
            " This can be used to force signing with consensus rules that are ahead of the node's current height.\n"

            "\nResult:\n"
            "{\n"
            "  \"hex\" : \"value\",           (string) The hex-encoded raw transaction with signature(s)\n"
            "  \"complete\" : true|false,   (boolean) If the transaction has a complete set of signatures\n"
            "  \"errors\" : [                 (json array of objects) Script verification errors (if there are any)\n"
            "    {\n"
            "      \"txid\" : \"hash\",           (string) The hash of the referenced, previous transaction\n"
            "      \"vout\" : n,                (numeric) The index of the output to spent and used as input\n"
            "      \"scriptSig\" : \"hex\",       (string) The hex-encoded signature script\n"
            "      \"sequence\" : n,            (numeric) Script sequence number\n"
            "      \"error\" : \"text\"           (string) Verification or signing error related to the input\n"
            "    }\n"
            "    ,...\n"
            "  ]\n"
            "}\n"

            "\nExamples:\n"
            + HelpExampleCli("signrawtransaction", "\"myhex\"")
            + HelpExampleRpc("signrawtransaction", "\"myhex\"")
        );

#ifdef ENABLE_WALLET
    LOCK2(cs_main, pwalletMain ? &pwalletMain->cs_wallet : NULL);
#else
    LOCK(cs_main);
#endif
    RPCTypeCheck(params, boost::assign::list_of(UniValue::VSTR)(UniValue::VARR)(UniValue::VARR)(UniValue::VSTR)(UniValue::VSTR), true);

    vector<unsigned char> txData(ParseHexV(params[0], "argument 1"));
    CDataStream ssData(txData, SER_NETWORK, PROTOCOL_VERSION);
    vector<CMutableTransaction> txVariants;
    while (!ssData.empty()) {
        try {
            CMutableTransaction tx;
            ssData >> tx;
            txVariants.push_back(tx);
        }
        catch (const std::exception&) {
            throw JSONRPCError(RPC_DESERIALIZATION_ERROR, "TX decode failed");
        }
    }

    if (txVariants.empty())
        throw JSONRPCError(RPC_DESERIALIZATION_ERROR, "Missing transaction");

    // mergedTx will end up with all the signatures; it
    // starts as a clone of the rawtx:
    CMutableTransaction mergedTx(txVariants[0]);

    // Fetch previous transactions (inputs):
    CCoinsView viewDummy;
    CCoinsViewCache view(&viewDummy);
    {
        LOCK(mempool.cs);
        CCoinsViewCache &viewChain = *pcoinsTip;
        CCoinsViewMemPool viewMempool(&viewChain, mempool);
        view.SetBackend(viewMempool); // temporarily switch cache backend to db+mempool view

        for (const CTxIn& txin : mergedTx.vin) {
            const uint256& prevHash = txin.prevout.hash;
            CCoins coins;
            view.AccessCoins(prevHash); // this is certainly allowed to fail
        }

        view.SetBackend(viewDummy); // switch back to avoid locking mempool for too long
    }

    KeyIO keyIO(Params());

    bool fGivenKeys = false;
    CBasicKeyStore tempKeystore;
    if (params.size() > 2 && !params[2].isNull()) {
        fGivenKeys = true;
        UniValue keys = params[2].get_array();
        for (size_t idx = 0; idx < keys.size(); idx++) {
            UniValue k = keys[idx];
            CKey key = keyIO.DecodeSecret(k.get_str());
            if (!key.IsValid())
                throw JSONRPCError(RPC_INVALID_ADDRESS_OR_KEY, "Invalid private key");
            tempKeystore.AddKey(key);
        }
    }
#ifdef ENABLE_WALLET
    else if (pwalletMain)
        EnsureWalletIsUnlocked();
#endif

    // Add previous txouts given in the RPC call:
    if (params.size() > 1 && !params[1].isNull()) {
        UniValue prevTxs = params[1].get_array();
        for (size_t idx = 0; idx < prevTxs.size(); idx++) {
            const UniValue& p = prevTxs[idx];
            if (!p.isObject())
                throw JSONRPCError(RPC_DESERIALIZATION_ERROR, "expected object with {\"txid'\",\"vout\",\"scriptPubKey\"}");

            UniValue prevOut = p.get_obj();

            RPCTypeCheckObj(prevOut, boost::assign::map_list_of("txid", UniValue::VSTR)("vout", UniValue::VNUM)("scriptPubKey", UniValue::VSTR));

            uint256 txid = ParseHashO(prevOut, "txid");

            int nOut = find_value(prevOut, "vout").get_int();
            if (nOut < 0)
                throw JSONRPCError(RPC_DESERIALIZATION_ERROR, "vout must be positive");

            vector<unsigned char> pkData(ParseHexO(prevOut, "scriptPubKey"));
            CScript scriptPubKey(pkData.begin(), pkData.end());

            {
                CCoinsModifier coins = view.ModifyCoins(txid);
                if (coins->IsAvailable(nOut) && coins->vout[nOut].scriptPubKey != scriptPubKey) {
                    string err("Previous output scriptPubKey mismatch:\n");
                    err = err + ScriptToAsmStr(coins->vout[nOut].scriptPubKey) + "\nvs:\n"+
                        ScriptToAsmStr(scriptPubKey);
                    throw JSONRPCError(RPC_DESERIALIZATION_ERROR, err);
                }
                if ((unsigned int)nOut >= coins->vout.size())
                    coins->vout.resize(nOut+1);
                coins->vout[nOut].scriptPubKey = scriptPubKey;
                coins->vout[nOut].nValue = 0;
                if (prevOut.exists("amount")) {
                    coins->vout[nOut].nValue = AmountFromValue(find_value(prevOut, "amount"));
                }
            }

            // if redeemScript given and not using the local wallet (private keys
            // given), add redeemScript to the tempKeystore so it can be signed:
            if (fGivenKeys && scriptPubKey.IsPayToScriptHash()) {
                RPCTypeCheckObj(prevOut, boost::assign::map_list_of("txid", UniValue::VSTR)("vout", UniValue::VNUM)("scriptPubKey", UniValue::VSTR)("redeemScript",UniValue::VSTR));
                UniValue v = find_value(prevOut, "redeemScript");
                if (!v.isNull()) {
                    vector<unsigned char> rsData(ParseHexV(v, "redeemScript"));
                    CScript redeemScript(rsData.begin(), rsData.end());
                    tempKeystore.AddCScript(redeemScript);
                }
            }
        }
    }

#ifdef ENABLE_WALLET
    const CKeyStore& keystore = ((fGivenKeys || !pwalletMain) ? tempKeystore : *pwalletMain);
#else
    const CKeyStore& keystore = tempKeystore;
#endif

    int nHashType = SIGHASH_ALL;
    if (params.size() > 3 && !params[3].isNull()) {
        static map<string, int> mapSigHashValues =
            boost::assign::map_list_of
            (string("ALL"), int(SIGHASH_ALL))
            (string("ALL|ANYONECANPAY"), int(SIGHASH_ALL|SIGHASH_ANYONECANPAY))
            (string("NONE"), int(SIGHASH_NONE))
            (string("NONE|ANYONECANPAY"), int(SIGHASH_NONE|SIGHASH_ANYONECANPAY))
            (string("SINGLE"), int(SIGHASH_SINGLE))
            (string("SINGLE|ANYONECANPAY"), int(SIGHASH_SINGLE|SIGHASH_ANYONECANPAY))
            ;
        string strHashType = params[3].get_str();
        if (mapSigHashValues.count(strHashType))
            nHashType = mapSigHashValues[strHashType];
        else
            throw JSONRPCError(RPC_INVALID_PARAMETER, "Invalid sighash param");
    }

    bool fHashSingle = ((nHashType & ~SIGHASH_ANYONECANPAY) == SIGHASH_SINGLE);
    // Use the approximate release height if it is greater so offline nodes
    // have a better estimation of the current height and will be more likely to
    // determine the correct consensus branch ID.  Regtest mode ignores release height.
    int chainHeight = chainActive.Height() + 1;
    if (Params().NetworkIDString() != "regtest") {
        chainHeight = std::max(chainHeight, APPROX_RELEASE_HEIGHT);
    }
    // Grab the current consensus branch ID
    auto consensusBranchId = CurrentEpochBranchId(chainHeight, Params().GetConsensus());

    if (params.size() > 4 && !params[4].isNull()) {
        consensusBranchId = ParseHexToUInt32(params[4].get_str());
        if (!IsConsensusBranchId(consensusBranchId)) {
            throw runtime_error(params[4].get_str() + " is not a valid consensus branch id");
        }
    }

    std::vector<CTxOut> allPrevOutputs;
    // We do not need to know the inputs for pre-v5 transactions.
    // We can't sign v5+ transactions without knowing all inputs.
    if (mergedTx.nVersion >= ZIP225_TX_VERSION) {
        if (!view.HaveInputs(mergedTx)) {
            throw JSONRPCError(RPC_INVALID_PARAMETER, "Cannot sign v5 transactions without knowing all inputs");
        }
        for (const auto& input : mergedTx.vin) {
            allPrevOutputs.push_back(view.GetOutputFor(input));
        }
    }

    // Script verification errors
    UniValue vErrors(UniValue::VARR);

    // Use CTransaction for the constant parts of the
    // transaction to avoid rehashing.
    const CTransaction txConst(mergedTx);
    const PrecomputedTransactionData txdata(txConst, allPrevOutputs);
    // Sign what we can:
    for (unsigned int i = 0; i < mergedTx.vin.size(); i++) {
        CTxIn& txin = mergedTx.vin[i];
        const CCoins* coins = view.AccessCoins(txin.prevout.hash);
        if (coins == NULL || !coins->IsAvailable(txin.prevout.n)) {
            TxInErrorToJSON(txin, vErrors, "Input not found or already spent");
            continue;
        }
        const CScript& prevPubKey = coins->vout[txin.prevout.n].scriptPubKey;
        const CAmount& amount = coins->vout[txin.prevout.n].nValue;

        SignatureData sigdata;
        // Only sign SIGHASH_SINGLE if there's a corresponding output:
        if (!fHashSingle || (i < mergedTx.vout.size()))
            ProduceSignature(MutableTransactionSignatureCreator(&keystore, &mergedTx, txdata, i, amount, nHashType), prevPubKey, sigdata, consensusBranchId);

        // ... and merge in other signatures:
        for (const CMutableTransaction& txv : txVariants) {
            sigdata = CombineSignatures(prevPubKey, TransactionSignatureChecker(&txConst, txdata, i, amount), sigdata, DataFromTransaction(txv, i), consensusBranchId);
        }

        UpdateTransaction(mergedTx, i, sigdata);

        ScriptError serror = SCRIPT_ERR_OK;
        if (!VerifyScript(txin.scriptSig, prevPubKey, STANDARD_SCRIPT_VERIFY_FLAGS, TransactionSignatureChecker(&txConst, txdata, i, amount), consensusBranchId, &serror)) {
            TxInErrorToJSON(txin, vErrors, ScriptErrorString(serror));
        }
    }
    bool fComplete = vErrors.empty();

    UniValue result(UniValue::VOBJ);
    result.pushKV("hex", EncodeHexTx(mergedTx));
    result.pushKV("complete", fComplete);
    if (!vErrors.empty()) {
        result.pushKV("errors", vErrors);
    }

    return result;
}

UniValue sendrawtransaction(const UniValue& params, bool fHelp)
{
    if (fHelp || params.size() < 1 || params.size() > 2)
        throw runtime_error(
            "sendrawtransaction \"hexstring\" ( allowhighfees )\n"
            "\nSubmits raw transaction (serialized, hex-encoded) to local node and network.\n"
            "\nAlso see createrawtransaction and signrawtransaction calls.\n"
            "\nArguments:\n"
            "1. \"hexstring\"    (string, required) The hex string of the raw transaction)\n"
            "2. allowhighfees    (boolean, optional, default=false) Allow high fees\n"
            "\nResult:\n"
            "\"hex\"             (string) The transaction hash in hex\n"
            "\nExamples:\n"
            "\nCreate a transaction\n"
            + HelpExampleCli("createrawtransaction", "\"[{\\\"txid\\\" : \\\"mytxid\\\",\\\"vout\\\":0}]\" \"{\\\"myaddress\\\":0.01}\"") +
            "Sign the transaction, and get back the hex\n"
            + HelpExampleCli("signrawtransaction", "\"myhex\"") +
            "\nSend the transaction (signed hex)\n"
            + HelpExampleCli("sendrawtransaction", "\"signedhex\"") +
            "\nAs a json rpc call\n"
            + HelpExampleRpc("sendrawtransaction", "\"signedhex\"")
        );

    LOCK(cs_main);
    RPCTypeCheck(params, boost::assign::list_of(UniValue::VSTR)(UniValue::VBOOL));

    // parse hex string from parameter
    CTransaction tx;
    if (!DecodeHexTx(tx, params[0].get_str()))
        throw JSONRPCError(RPC_DESERIALIZATION_ERROR, "TX decode failed");
    uint256 hashTx = tx.GetHash();

    auto chainparams = Params();

    // DoS mitigation: reject transactions expiring soon
    if (tx.nExpiryHeight > 0) {
        int nextBlockHeight = chainActive.Height() + 1;
        if (chainparams.GetConsensus().NetworkUpgradeActive(nextBlockHeight, Consensus::UPGRADE_OVERWINTER)) {
            if (nextBlockHeight + TX_EXPIRING_SOON_THRESHOLD > tx.nExpiryHeight) {
                throw JSONRPCError(RPC_TRANSACTION_REJECTED,
                    strprintf("tx-expiring-soon: expiryheight is %d but should be at least %d to avoid transaction expiring soon",
                    tx.nExpiryHeight,
                    nextBlockHeight + TX_EXPIRING_SOON_THRESHOLD));
            }
        }
    }

    bool fOverrideFees = false;
    if (params.size() > 1)
        fOverrideFees = params[1].get_bool();

    CCoinsViewCache &view = *pcoinsTip;
    const CCoins* existingCoins = view.AccessCoins(hashTx);
    bool fHaveMempool = mempool.exists(hashTx);
    bool fHaveChain = existingCoins && existingCoins->nHeight < 1000000000;
    if (!fHaveMempool && !fHaveChain) {
        // push to local node and sync with wallets
        CValidationState state;
        bool fMissingInputs;
        if (!AcceptToMemoryPool(chainparams, mempool, state, tx, false, &fMissingInputs, !fOverrideFees)) {
            if (state.IsInvalid()) {
                throw JSONRPCError(RPC_TRANSACTION_REJECTED, strprintf("%i: %s", state.GetRejectCode(), state.GetRejectReason()));
            } else {
                if (fMissingInputs) {
                    throw JSONRPCError(RPC_TRANSACTION_ERROR, "Missing inputs");
                }
                throw JSONRPCError(RPC_TRANSACTION_ERROR, state.GetRejectReason());
            }
        }
    } else if (fHaveChain) {
        throw JSONRPCError(RPC_TRANSACTION_ALREADY_IN_CHAIN, "transaction already in block chain");
    }
    RelayTransaction(tx);

    return hashTx.GetHex();
}

static const CRPCCommand commands[] =
{ //  category              name                      actor (function)         okSafeMode
  //  --------------------- ------------------------  -----------------------  ----------
    { "rawtransactions",    "getrawtransaction",      &getrawtransaction,      true  },
    { "rawtransactions",    "createrawtransaction",   &createrawtransaction,   true  },
    { "rawtransactions",    "decoderawtransaction",   &decoderawtransaction,   true  },
    { "rawtransactions",    "decodescript",           &decodescript,           true  },
    { "rawtransactions",    "sendrawtransaction",     &sendrawtransaction,     false },
    { "rawtransactions",    "signrawtransaction",     &signrawtransaction,     false }, /* uses wallet if enabled */

    { "blockchain",         "gettxoutproof",          &gettxoutproof,          true  },
    { "blockchain",         "verifytxoutproof",       &verifytxoutproof,       true  },
};

void RegisterRawTransactionRPCCommands(CRPCTable &tableRPC)
{
    for (unsigned int vcidx = 0; vcidx < ARRAYLEN(commands); vcidx++)
        tableRPC.appendCommand(commands[vcidx].name, &commands[vcidx]);
}
