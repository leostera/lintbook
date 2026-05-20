-- Test cases for SQL013: SP_ prefix violations (T-SQL specific)

-- Violations: User procedures with SP_ prefix
CREATE PROCEDURE sp_GetUserData
    @UserId INT
AS
BEGIN
    SELECT * FROM Users WHERE Id = @UserId;
END;

-- Violations: ALTER PROCEDURE with SP_ prefix
ALTER PROCEDURE sp_UpdateUserStatus
    @UserId INT,
    @Status VARCHAR(20)
AS
BEGIN
    UPDATE Users SET Status = @Status WHERE Id = @UserId;
END;

-- Violations: Multiple procedures with SP_ prefix
CREATE PROC sp_DeleteUser @UserId INT
AS
BEGIN
    DELETE FROM Users WHERE Id = @UserId;
END;

ALTER PROC sp_CreateUser 
    @Name VARCHAR(100),
    @Email VARCHAR(100)
AS
BEGIN
    INSERT INTO Users (Name, Email) VALUES (@Name, @Email);
END;

-- OK: Proper user procedure names
CREATE PROCEDURE GetUserData
    @UserId INT
AS
BEGIN
    SELECT * FROM Users WHERE Id = @UserId;
END;

-- OK: System procedures (these would be built-in, not user-defined)
-- EXEC sp_helpdb;  -- This is a system procedure, not user-defined

CREATE PROCEDURE usp_GetUserData  -- Common prefix for user procedures
    @UserId INT  
AS
BEGIN
    SELECT * FROM Users WHERE Id = @UserId;
END;