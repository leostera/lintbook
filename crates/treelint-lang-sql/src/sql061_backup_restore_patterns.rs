use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct BackupRestorePatterns;

impl Rule for BackupRestorePatterns {
    fn id(&self) -> &'static str {
        "SQL061"
    }

    fn name(&self) -> &'static str {
        "backup-restore-patterns"
    }

    fn description(&self) -> &'static str {
        "Enforce backup and restore operation best practices"
    }

    fn explanation(&self) -> &'static str {
        "Backup and restore operations are critical for data protection. This rule checks for
        proper backup strategies, restore considerations, and maintenance best practices."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_backup_restore_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl BackupRestorePatterns {
    fn check_backup_restore_patterns(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            
            // Skip comments
            if line.trim().starts_with("--") {
                continue;
            }
            
            // Check backup operations
            if lower_line.contains("backup database") {
                self.check_backup_operations(&lower_line, line_idx, node, violations);
            }
            
            // Check restore operations
            if lower_line.contains("restore database") || lower_line.contains("restore filelistonly") {
                self.check_restore_operations(&lower_line, line_idx, node, violations);
            }
            
            // Check maintenance operations
            if lower_line.contains("dbcc") {
                self.check_dbcc_operations(&lower_line, line_idx, node, violations);
            }
            
            // Check log backup patterns
            if lower_line.contains("backup log") {
                self.check_log_backup_patterns(&lower_line, line_idx, node, violations);
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_backup_restore_patterns(child, source, violations);
            }
        }
    }
    
    fn check_backup_operations(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for backup without compression
        if !lower_line.contains("compression") && !lower_line.contains("compress") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Database backup without compression. Consider WITH COMPRESSION for reduced backup size".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for backup without verification
        if !lower_line.contains("checksum") && !lower_line.contains("verify") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Database backup without checksum verification. Consider WITH CHECKSUM for backup integrity".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for backup to single file (availability risk)
        if !lower_line.contains(",") && lower_line.contains("to disk") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Backup to single file. Consider multiple backup files for faster restore and redundancy".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for backup without proper naming convention
        if lower_line.contains("to disk") {
            if !lower_line.contains(".bak") && !lower_line.contains(".backup") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Backup file without standard extension (.bak). Use consistent naming for easier management".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
            
            // Check for timestamp in backup name
            if !lower_line.contains("2023") && !lower_line.contains("2024") && !lower_line.contains("202") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Backup filename without timestamp. Include date/time for better backup management".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for backup without copy_only (for ad-hoc backups)
        if lower_line.contains("backup database") && !lower_line.contains("copy_only") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Full backup without COPY_ONLY. Consider COPY_ONLY for ad-hoc backups to avoid breaking backup chain".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_restore_operations(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for restore without verification
        if lower_line.contains("restore database") && !lower_line.contains("checksum") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Database restore without checksum verification. Consider WITH CHECKSUM for restore integrity".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for restore without norecovery (for log shipping)
        if lower_line.contains("restore database") && !lower_line.contains("recovery") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Database restore without recovery option specified. Be explicit about RECOVERY/NORECOVERY".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for restore to same location (overwrite risk)
        if lower_line.contains("restore database") && !lower_line.contains("move") && !lower_line.contains("replace") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Database restore without MOVE or REPLACE. Ensure target location is correct".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for restore without stats (progress monitoring)
        if lower_line.contains("restore database") && !lower_line.contains("stats") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Database restore without STATS option. Consider STATS for progress monitoring".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_dbcc_operations(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check DBCC CHECKDB usage
        if lower_line.contains("dbcc checkdb") {
            if !lower_line.contains("with no_infomsgs") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "DBCC CHECKDB without NO_INFOMSGS. Consider adding to reduce output volume".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
            
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DBCC CHECKDB detected. Ensure this runs during maintenance windows due to resource usage".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check DBCC SHRINKFILE usage (generally discouraged)
        if lower_line.contains("dbcc shrinkfile") || lower_line.contains("dbcc shrinkdatabase") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DBCC SHRINK operation detected. This can cause fragmentation and performance issues".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check DBCC maintenance operations
        if lower_line.contains("dbcc dbreindex") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DBCC DBREINDEX is deprecated. Use ALTER INDEX REBUILD for modern SQL Server versions".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        if lower_line.contains("dbcc indexdefrag") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DBCC INDEXDEFRAG is deprecated. Use ALTER INDEX REORGANIZE for modern SQL Server versions".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_log_backup_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for log backup without compression
        if !lower_line.contains("compression") && !lower_line.contains("compress") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Transaction log backup without compression. Consider WITH COMPRESSION for reduced backup size".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for log backup with wrong file extension
        if lower_line.contains("to disk") && !lower_line.contains(".trn") && !lower_line.contains(".log") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Log backup without standard extension (.trn or .log). Use consistent naming for easier management".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for log backup in simple recovery model
        let start_pos = node.start_position();
        violations.push(LintViolation {
            line: start_pos.row + line_idx + 1,
            column: start_pos.column + 1,
            message: "Transaction log backup detected. Ensure database is in FULL or BULK_LOGGED recovery model".to_string(),
            lint_name: self.name().to_string(),
            lint_id: self.id().to_string(),
        });
    }
}