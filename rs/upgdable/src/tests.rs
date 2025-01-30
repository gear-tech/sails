use super::*;

mod program_v11 {
    use sails_rs::ActorId;

    use crate::program_v11::*;

    #[test]
    fn test_migration() {
        let source_actor_id = ActorId::zero();
        let mut migration = MigrationImpl;
        migrate(source_actor_id, &mut migration);
    }
}
