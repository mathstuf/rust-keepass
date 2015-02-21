use super::v1kpdb::V1Kpdb;
use super::v1error::V1KpdbError;
use super::v1header::V1Header;
use sec_str::SecureString;

#[test]
fn test_new() {
    // No keyfile and password should give error as result
    let mut result = V1Kpdb::new("test/test_password.kdb".to_string(), None, None);
    match result {
        Ok(_)  => assert!(false),
        Err(e) => assert_eq!(e, V1KpdbError::PassErr),
    };

    // Test load at all and parameters
    result = V1Kpdb::new("test/test_both.kdb".to_string(), Some("test".to_string()),
                         Some("test/test_key".to_string()));
    assert!(result.is_ok());
    let mut db = result.ok().unwrap();
    assert_eq!(db.load().is_ok(), true);
    assert_eq!(db.path.as_slice(), "test/test_both.kdb");

    match db.password {
        Some(mut s) => {assert_eq!(s.string.as_slice(), "\0\0\0\0");
                        s.unlock();
                        assert_eq!(s.string.as_slice(), "test")},
        None => assert!(false),
    };
    match db.keyfile {
        Some(mut s) => {assert_eq!(s.string.as_slice(), "\0\0\0\0\0\0\0\0\0\0\0\0\0");
                        s.unlock();
                        assert_eq!(s.string.as_slice(), "test/test_key")},
        None => assert!(false),
    };
    
    // Test fail of load with wrong password
    result = V1Kpdb::new("test/test_password.kdb".to_string(), Some("tes".to_string()), None);
    assert!(result.is_ok());
    db = result.ok().unwrap();
    match db.load() {
        Ok(_)  => assert!(false),
        Err(e) => assert_eq!(e, V1KpdbError::HashErr),
    };
}


#[test]
fn test_parse_groups () {
    let mut header = V1Header::new();
    let _ = header.read_header("test/test_password.kdb".to_string());
    let sec_str = SecureString::new("test".to_string());
    let mut keyfile = None;
    let mut decrypted_database: Vec<u8> = vec![];
    match V1Kpdb::decrypt_database("test/test_password.kdb".to_string(),
                                   &mut Some(sec_str), &mut keyfile,
                                   &header) {
        Ok(e)  => { decrypted_database = e;},
        Err(_) => assert!(false),
    };
    
    let mut groups = vec![];
    match V1Kpdb::parse_groups(&header, &decrypted_database, &mut 0usize) {
        Ok((e, _)) => { groups = e; }, 
        Err(_) => assert!(false),
    }

    assert_eq!(groups[0].borrow().id, 1);
    assert_eq!(groups[0].borrow().title.as_slice(), "Internet");
    assert_eq!(groups[0].borrow().image, 1);
    assert_eq!(groups[0].borrow().level, 0);
    assert_eq!(groups[0].borrow().creation.year, 0);
    assert_eq!(groups[0].borrow().creation.month, 0);
    assert_eq!(groups[0].borrow().creation.day, 0);

    assert_eq!(groups[1].borrow().id, 2);
    assert_eq!(groups[1].borrow().title.as_slice(), "test");
    assert_eq!(groups[1].borrow().image, 1);
    assert_eq!(groups[1].borrow().level, 0);
    assert_eq!(groups[1].borrow().creation.year, 2014);
    assert_eq!(groups[1].borrow().creation.month, 2);
    assert_eq!(groups[1].borrow().creation.day, 26);
}

#[test]
fn test_parse_entries () {
    let uuid: Vec<u8> = vec![0x0c, 0x31, 0xac, 0x94, 0x23, 0x47, 0x66, 0x36, 
                             0xb8, 0xc0, 0x42, 0x81, 0x5e, 0x5a, 0x14, 0x60];

    let mut header = V1Header::new();
    let _ = header.read_header("test/test_password.kdb".to_string());
    let sec_str = SecureString::new("test".to_string());
    let mut keyfile = None;
    let mut decrypted_database: Vec<u8> = vec![];
    match V1Kpdb::decrypt_database("test/test_password.kdb".to_string(),
                                   &mut Some(sec_str), &mut keyfile,
                                   &header) {
        Ok(e)  => { decrypted_database = e;},
        Err(_) => assert!(false),
    };

    let mut entries = vec![];
    match V1Kpdb::parse_entries(&header, &decrypted_database, &mut 138usize) {
        Ok(e)  => { entries = e; }, 
        Err(_) => assert!(false),
    }

    entries[0].borrow_mut().password.unlock();

    assert_eq!(entries[0].borrow().uuid, uuid);
    assert_eq!(entries[0].borrow().title.as_slice(), "foo");
    assert_eq!(entries[0].borrow().url.as_slice(), "foo");
    assert_eq!(entries[0].borrow().username.as_slice(), "foo");
    assert_eq!(entries[0].borrow().password.string.as_slice(), "DLE\"H<JZ|E");
    assert_eq!(entries[0].borrow().image, 1);
    assert_eq!(entries[0].borrow().group_id, 1);
    assert_eq!(entries[0].borrow().creation.year, 2014);
    assert_eq!(entries[0].borrow().creation.month, 2);
    assert_eq!(entries[0].borrow().creation.day, 26);
}

#[test]
fn test_create_group_tree() {
    let mut db = V1Kpdb::new("test/test_parsing.kdb".to_string(),
                             Some("test".to_string()), None).ok().unwrap();
    
    let mut header = V1Header::new();
    let _ = header.read_header("test/test_parsing.kdb".to_string());
    let sec_str = SecureString::new("test".to_string());
    let mut keyfile = None;
    let mut decrypted_database: Vec<u8> = vec![];
    match V1Kpdb::decrypt_database("test/test_parsing.kdb".to_string(),
                                   &mut Some(sec_str), &mut keyfile,
                                   &header) {
        Ok(e)  => { decrypted_database = e;},
        Err(_) => assert!(false),
    };

    let mut pos = 0usize;

    let mut groups = vec![];
    let mut levels = vec![];
    match V1Kpdb::parse_groups(&header, &decrypted_database, &mut pos) {
        Ok((e, l)) => { groups = e;
                        levels = l;}, 
        Err(_) => assert!(false),
    }
    db.groups = groups;

    let mut pos_cpy = pos;
    let mut entries = vec![];
    match V1Kpdb::parse_entries(&header, &decrypted_database, &mut pos_cpy) {
        Ok(e)  => { entries = e; }, 
        Err(_) => assert!(false),
    }
    db.entries = entries;

    assert_eq!(V1Kpdb::create_group_tree(&mut db, levels).is_ok(), true);


    let mut group = db.groups[1].borrow_mut();
    let parent = group.parent.as_mut().unwrap().borrow();
    let parent_title = parent.title.as_slice();
    assert_eq!(parent_title, "Internet");
    // assert_eq!(db.groups[2].borrow_mut().parent.as_mut()
    //            .unwrap().borrow().title.as_slice(), "Internet");
    // assert_eq!(db.groups[2].borrow_mut().children[0]
    //            .upgrade().unwrap().borrow().title.as_slice(), "22");
    // assert_eq!(db.groups[2].borrow_mut().children[1]
    //            .upgrade().unwrap().borrow().title.as_slice(), "21");
    // assert_eq!(db.groups[3].borrow_mut().parent.as_mut()
    //            .unwrap().borrow().title.as_slice(), "11");
    // assert_eq!(db.groups[4].borrow_mut().parent.as_mut()
    //            .unwrap().borrow().title.as_slice(), "11");
    // assert_eq!(db.groups[4].borrow_mut().children[0]
    //            .upgrade().unwrap().borrow().title.as_slice(), "32");
    // assert_eq!(db.groups[4].borrow_mut().children[1]
    //            .upgrade().unwrap().borrow().title.as_slice(), "31");
    // assert_eq!(db.groups[5].borrow_mut().parent.as_mut()
    //            .unwrap().borrow().title.as_slice(), "21");
    // assert_eq!(db.groups[6].borrow_mut().parent.as_mut()
    //            .unwrap().borrow().title.as_slice(), "21");

    // assert_eq!(db.entries[0].borrow_mut().group.as_mut()
    //            .unwrap().borrow().title.as_slice(), "Internet");
    // assert_eq!(db.entries[1].borrow_mut().group.as_mut()
    //            .unwrap().borrow().title.as_slice(), "11");
    // assert_eq!(db.entries[2].borrow_mut().group.as_mut()
    //            .unwrap().borrow().title.as_slice(), "12");
    // assert_eq!(db.entries[3].borrow_mut().group.as_mut()
    //            .unwrap().borrow().title.as_slice(), "21");
    // assert_eq!(db.entries[4].borrow_mut().group.as_mut()
    //            .unwrap().borrow().title.as_slice(), "22");
}
