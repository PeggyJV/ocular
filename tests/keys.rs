#[cfg(feature = "keys")]

#[test]
fn account_from_pem() {
    use std::fs;

    use k256::{ecdsa::SigningKey, elliptic_curve::SecretKey, Secp256k1};
    use ocular::prelude::AccountInfo;
    use pkcs8::EncodePrivateKey;
    use rand_core::OsRng;

    // generate new key
    let key = SigningKey::random(&mut OsRng);
    let key = SecretKey::<Secp256k1>::from(key);
    let pem = key.to_pkcs8_pem(Default::default()).unwrap();

    // write the key as a PEM-encoded file
    let pem_path = "./test.pem";
    let _ = fs::remove_file(pem_path);
    fs::write(pem_path, pem.as_bytes()).unwrap();

    // test constructor
    let _pem_account = AccountInfo::from_pkcs8_pem(pem_path).unwrap();

    // clean up the key
    fs::remove_file(pem_path).unwrap();
}

#[test]
fn account_from_encrypted_pem() {
    use std::fs;

    use k256::{ecdsa::SigningKey, elliptic_curve::SecretKey, Secp256k1};
    use ocular::prelude::AccountInfo;
    use pkcs8::EncodePrivateKey;
    use rand_core::OsRng;

    // generate new key
    let key = SigningKey::random(&mut OsRng);
    let key = SecretKey::<Secp256k1>::from(key);
    let pem = key.to_pkcs8_encrypted_pem(&mut OsRng, "password".as_bytes(), Default::default()).unwrap();

    // write the key as a PEM-encoded file
    let pem_path = "./test.pem";
    let _ = fs::remove_file(pem_path);
    fs::write(pem_path, pem.as_bytes()).unwrap();

    // test constructor
    let _pem_account = AccountInfo::from_pkcs8_encrypted_pem(pem_path, "password").unwrap();

    // clean up the key
    fs::remove_file(pem_path).unwrap();
}

#[test]
fn account_from_der() {
    use std::fs;

    use k256::{ecdsa::SigningKey, elliptic_curve::SecretKey, Secp256k1};
    use ocular::prelude::AccountInfo;
    use pkcs8::EncodePrivateKey;
    use rand_core::OsRng;

    // generate new key
    let key = SigningKey::random(&mut OsRng);
    let key = SecretKey::<Secp256k1>::from(key);
    let der = key.to_pkcs8_der().unwrap();

    // write the key as a DER-encoded file
    let der_path = "./test_encrypted.der";
    let _ = fs::remove_file(der_path);
    fs::write(der_path, der.as_bytes()).unwrap();

    // test constructor
    let _der_account = AccountInfo::from_pkcs8_der(der_path).unwrap();

    // clean up key
    fs::remove_file(der_path).unwrap();
}

#[test]
fn account_from_encrypted_der() {
    use std::fs;

    use k256::{ecdsa::SigningKey, elliptic_curve::SecretKey, Secp256k1};
    use ocular::prelude::AccountInfo;
    use pkcs8::EncodePrivateKey;
    use rand_core::OsRng;

    // generate new key
    let key = SigningKey::random(&mut OsRng);
    let key = SecretKey::<Secp256k1>::from(key);
    let der = key.to_pkcs8_encrypted_der(&mut OsRng, "password".as_bytes()).unwrap();

    // write the key as a DER-encoded file
    let der_path = "./test_encrypted.der";
    let _ = fs::remove_file(der_path);
    fs::write(der_path, der.as_bytes()).unwrap();

    // test constructor
    let _der_account = AccountInfo::from_pkcs8_encrypted_der(der_path, "password").unwrap();

    // clean up key
    fs::remove_file(der_path).unwrap();
}
