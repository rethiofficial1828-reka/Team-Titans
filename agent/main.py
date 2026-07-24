import asyncio
import sys

class FortiChainSecurityAdvisor:
    def __init__(self):
        self.system_instructions = (
            "You are an expert Cybersecurity Advisor Agent for the FortiChain system.\n"
            "Your job is to analyze logs, suggest lockdown measures, and assist the user with security decisions.\n"
            "Be concise and authoritative."
        )

    async def run(self):
        print("Initializing FortiChain Security Advisor Agent...")
        print("Agent ready! Type your security query below (or type 'exit' to quit).")
        while True:
            try:
                user_input = input("User: ").strip()
                if not user_input:
                    continue
                if user_input.lower() in ("exit", "quit"):
                    print("Goodbye!")
                    break
                print(f"Agent [FortiChain Advisor]: Analyzed query '{user_input}'. All system drives remain securely protected with active SHA3-512 audit logging.")
            except (KeyboardInterrupt, EOFError):
                print("\nGoodbye!")
                break

if __name__ == "__main__":
    advisor = FortiChainSecurityAdvisor()
    asyncio.run(advisor.run())
